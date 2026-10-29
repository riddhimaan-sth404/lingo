use lingo_driver::Driver;
use std::fs;

#[test]
fn test_runtime_concurrency_channel() {
    let mut driver = Driver::new();
    let temp_dir = std::env::temp_dir().join("lingo_concurrency_tests");
    fs::create_dir_all(&temp_dir).unwrap();

    let src = "def main():\n    ch = Channel()\n    channel_send(ch, \"ping\")\n    msg = channel_recv(ch)\n    println(msg)\n";
    let test_file = temp_dir.join("test_chan.ln");
    fs::write(&test_file, src).unwrap();

    let exe_path = driver.build(&test_file, None).expect("Build should succeed");
    let output = std::process::Command::new(&exe_path).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "ping");

    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&exe_path);
}
