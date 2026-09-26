# Local Windows Remote Control Build

This fork contains a local Windows-specific fix for Codex Remote Control.

## Why this fix was started

Remote Control was needed to operate Codex safely from a phone while away from the computer. The Windows-specific startup failure blocked that workflow even though the local Codex installation and the user's project permissions were otherwise valid. The fix was therefore prepared as an isolated local build so current work could continue without changing the official Codex installation or any winwin.travel project.

## What changed

Foreground Remote Control now uses the platform temporary directory on Windows. Unix builds retain the existing `/tmp` preference. This avoids selecting a Windows path with incompatible socket-directory permissions while still using a private temporary directory.

The change is implemented in `codex-rs/cli/src/remote_control_cmd.rs` and includes a regression test named `foreground_socket_directory_uses_platform_temp_root`.

## Build and test

From `codex-rs`:

```powershell
cargo test -p codex-cli --bin codex foreground_socket_directory_uses_platform_temp_root -j 1
cargo build --release -p codex-cli -j 2
```

The release binary is written to:

```text
codex-rs/target/release/codex.exe
```

The `-j 2` option provides moderate parallelism. If the machine starts exhausting memory or the paging file, retry with `-j 1`.

## Upstream contribution status

The upstream repository documents that external pull requests are accepted by invitation only. For that reason, this work was kept in a personal fork and local branch, and the investigation and proposed fix were described in issue `openai/codex#47416` instead of opening an unsolicited pull request. The local patch is intentionally narrow and the focused regression test passes, but it may not be the final upstream-quality implementation; maintainer review is still required.

## Run the local build without replacing official Codex

Keep the official Codex installation on `PATH`. Start the fork explicitly:

```powershell
$localCodex = "C:\Users\anton\Projects\codex-fork\codex-rs\target\release\codex.exe"
& $localCodex remote-control
```

In a second non-elevated PowerShell window, request pairing:

```powershell
& $localCodex remote-control pair
```

The local binary is isolated from the official `codex` command. Returning to the official version requires no rollback: close the local process and run `codex` normally.

## Upstream tracking

The GitHub issue is `openai/codex#47416`. Issue notifications are enabled, and repository notifications are enabled for releases. When upstream ships an official fix, compare the release before switching back and remove this local override only after validating Remote Control.

This fork is not an official OpenAI distribution. Preserve the repository license and notices when redistributing it.
