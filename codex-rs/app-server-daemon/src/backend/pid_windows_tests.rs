use std::process::Stdio;

use pretty_assertions::assert_eq;
use tempfile::tempdir;
use tokio::fs::OpenOptions;

use super::PidRecord;
use super::process_matches_record;
use super::read_process_start_time;
use super::try_lock_file;

#[tokio::test]
async fn pid_lock_is_exclusive() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("daemon.pid.lock");
    let first = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .await
        .expect("first lock file");
    let second = OpenOptions::new()
        .write(true)
        .open(&path)
        .await
        .expect("second lock file");

    assert!(try_lock_file(&first).expect("first lock acquisition"));
    assert!(!try_lock_file(&second).expect("second lock acquisition"));
}

#[tokio::test]
async fn process_start_time_rejects_pid_reuse_record() {
    let mut child = std::process::Command::new("cmd.exe")
        .args(["/C", "ping", "-n", "20", "127.0.0.1"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn test process");
    let pid = child.id();
    let process_start_time = read_process_start_time(pid).await.expect("start time");
    let record = PidRecord {
        pid,
        process_start_time: process_start_time.clone(),
    };
    let reused_pid_record = PidRecord {
        pid,
        process_start_time: format!("{process_start_time}-reused"),
    };

    assert_eq!(
        process_matches_record(&record)
            .await
            .expect("matching record"),
        true
    );
    assert_eq!(
        process_matches_record(&reused_pid_record)
            .await
            .expect("reused pid record"),
        false
    );

    child.kill().expect("terminate test process");
    child.wait().expect("reap test process");
}
