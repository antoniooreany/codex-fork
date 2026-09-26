use pretty_assertions::assert_eq;

#[cfg(unix)]
use super::executable_identity_from_bytes;
use super::managed_codex_bin;
use super::parse_codex_version;

#[test]
fn parses_codex_cli_version_output() {
    assert_eq!(
        parse_codex_version("codex 1.2.3\n").expect("version"),
        "1.2.3"
    );
}

#[test]
fn rejects_malformed_codex_cli_version_output() {
    assert!(parse_codex_version("codex\n").is_err());
}

#[cfg(unix)]
#[test]
fn executable_identity_uses_binary_contents() {
    let old = executable_identity_from_bytes(b"old");
    let same = executable_identity_from_bytes(b"old");
    let new = executable_identity_from_bytes(b"new");

    assert_eq!(old, same);
    assert_ne!(old, new);
}

#[test]
fn managed_codex_path_uses_platform_install_layout() {
    let path = managed_codex_bin(std::path::Path::new("codex-home"));
    let mut expected = std::path::PathBuf::from("codex-home")
        .join("packages")
        .join("standalone")
        .join("current");
    if cfg!(windows) {
        expected.push("bin");
        expected.push("codex.exe");
    } else {
        expected.push("codex");
    }

    assert_eq!(path, expected);
}
