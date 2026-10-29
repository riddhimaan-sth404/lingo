use lingo_driver::Driver;
use std::fs;

#[test]
fn test_compile_and_run_arithmetic() {
    let mut driver = Driver::new();
    let temp_dir = std::env::temp_dir().join("lingo_tests");
    fs::create_dir_all(&temp_dir).unwrap();

    let src = "def main():\n    x = 10\n    y = 20\n    z = x + y * 2\n    println(z)\n";
    let test_file = temp_dir.join("test_arithmetic.ln");
    fs::write(&test_file, src).unwrap();

    let exe_path = driver.build(&test_file, None).expect("Build should succeed");
    assert!(exe_path.exists());

    let output = std::process::Command::new(&exe_path)
        .output()
        .expect("Execution should succeed");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "50");

    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&exe_path);
}

#[test]
fn test_compile_fail_undeclared_variable() {
    let mut driver = Driver::new();
    let temp_dir = std::env::temp_dir().join("lingo_tests");
    fs::create_dir_all(&temp_dir).unwrap();

    let src = "def main():\n    println(undeclared_var)\n";
    let test_file = temp_dir.join("test_fail.ln");
    fs::write(&test_file, src).unwrap();

    let res = driver.build(&test_file, None);
    assert!(res.is_err());

    let _ = fs::remove_file(&test_file);
}
