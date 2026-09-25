# Windows Remote Control Daemon Support

## Status

Draft for review. No implementation is included in this document.

## Motivation

The local Windows build now uses the platform temporary directory for the
foreground socket directory, but `codex remote-control` still fails before
starting because the app-server daemon lifecycle is guarded as Unix-only.
This blocks the intended workflow of running Codex on Windows and controlling
it remotely from a phone.

The change is isolated to the personal fork and must not alter the official
Codex installation or any winwin.travel project.

## Requirements

### Functional requirements

1. `codex remote-control start` must work on supported Windows x64 builds.
2. `codex remote-control stop` must stop only the Codex app-server process
   recorded by the local daemon state.
3. `codex remote-control restart` must stop and start the same managed
   instance without killing unrelated processes.
4. `codex remote-control pair` must work after a Windows daemon start.
5. Existing Unix lifecycle behavior must remain unchanged.
6. Stale PID records must be detected using both PID and process start time.
7. The implementation must keep the app-server control socket private and
   must not expose an unauthenticated network listener.

### Non-functional requirements

1. Use Gitflow: implementation on `fix/windows-remote-control-daemon`, with
   atomic commits and no direct work on `main` or `develop`.
2. Follow TDD: add Windows-focused tests before production implementation
   where practical.
3. Preserve the existing public CLI and JSON output shapes unless a Windows
   limitation requires a documented addition.
4. Keep Windows-specific code behind platform boundaries so Unix code remains
   easy to review.
5. Do not modify CI configuration merely to bypass failing checks.

## Proposed design

### Backend abstraction

Keep the existing daemon API and introduce a Windows implementation for the
currently Unix-only process lifecycle operations. The backend will:

- create and reserve the PID file under `CODEX_HOME/app-server-daemon`;
- launch the managed app-server with Windows process creation APIs;
- create a new process group/session so the child is not tied to the terminal;
- record PID and process start time;
- redirect stderr to the existing daemon log;
- poll readiness through the existing local control socket;
- stop the process only after validating the recorded PID/start time;
- use graceful termination first, followed by a bounded force-termination
  fallback.

The Unix implementation remains the reference path and retains its existing
`setsid`, signal, updater, and process-reaping behavior.

### Windows process safety

The implementation must use Windows APIs or the existing Rust Windows support
already present in the workspace. It must not use `taskkill /F` by PID without
first validating process identity. It must not open a public WebSocket as a
workaround. The app-server transport remains the existing private control
socket unless the upstream protocol explicitly requires otherwise.

### Lifecycle state

Reuse the existing PID record and lock-file model. Windows process start time
must use a stable system value that can be compared after PID reuse. Missing,
stale, starting, and running states retain their current semantics.

### Error handling

Replace the generic Unix-only error on Windows with actionable errors for:

- process creation failure;
- inability to create or reserve daemon state;
- readiness timeout;
- stale PID record;
- graceful stop timeout and force-stop result.

Error messages must not include credentials or unrelated command-line secrets.

## Testing strategy

### Unit tests

Add tests for:

- Windows command-line construction and remote-control environment flags;
- PID record serialization and process-start-time matching;
- stale PID rejection when a PID is reused;
- lifecycle state transitions for missing, starting, running, and stopped;
- graceful-stop timeout and bounded force-stop behavior;
- private socket-directory selection on Windows.

Tests that require Windows APIs must be guarded with `cfg(windows)` and run
on a Windows CI runner. Pure state-machine tests should run on every platform.

### Integration checks

On Windows, run:

```powershell
cargo test -p codex-app-server-daemon
cargo test -p codex-cli --bin codex
cargo build --release -p codex-cli -j 2
```

Then validate locally with a private `CODEX_HOME` test directory:

```powershell
codex remote-control start
codex remote-control pair
codex remote-control stop
```

The validation must confirm that no unrelated process is stopped and that the
official `codex` installation remains unchanged.

## CI/CD plan

1. Keep the fast Rust checks enabled for `codex-rs` changes.
2. Add or extend a Windows CI job for `codex-app-server-daemon` tests.
3. Reuse the repository's pinned Rust toolchain and existing Windows runner
   conventions from `rust-release-windows.yml`.
4. Run formatting, focused daemon tests, CLI tests, and a release build.
5. Keep the release workflow unchanged unless the new Windows test requires a
   narrowly scoped test-only step.
6. Do not publish a release artifact from the personal fork automatically.
7. Treat CI failure as a stop condition before merging or switching the local
   wrapper to the new binary.

## Gitflow and delivery

1. Current branch: `fix/windows-remote-control-daemon`.
2. Commit the SDD separately from implementation.
3. After SDD approval, write tests first, then implement the Windows backend.
4. Use focused atomic commits and run `git diff --check` before each commit.
5. Run the local CI-equivalent checks before considering the branch ready.
6. Keep the branch in the personal fork. Do not open an unsolicited upstream
   PR because the upstream contribution policy currently does not accept
   external code contributions.

## Acceptance criteria

The change is complete when all of the following are true:

- Windows `codex remote-control start` no longer returns the Unix-only error;
- `pair`, `stop`, and `restart` work against the same managed process;
- focused Windows tests pass;
- the release binary builds successfully;
- no unauthenticated network listener is introduced;
- Unix tests and behavior remain green;
- documentation explains Windows setup, limitations, rollback, and the local
  fork boundary.

## Risks and rollback

The main risks are PID reuse, process-tree leakage, Windows job-object rules,
and accidental termination of an unrelated process. The implementation must
fail closed when process identity cannot be verified.

Rollback is limited to stopping the local daemon, switching back to the
previous local binary, and leaving the official Codex installation untouched.
