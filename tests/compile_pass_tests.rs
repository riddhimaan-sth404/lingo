use lingo_driver::Driver;
use std::fs;

#[test]
fn test_compile_pass_while_and_functions() {
    let mut driver = Driver::new();
    let temp_dir = std::env::temp_dir().join("lingo_pass_tests");
    fs::create_dir_all(&temp_dir).unwrap();

    let src = "def square(x):\n    return x * x\n\ndef main():\n    val = square(7)\n    println(val)\n";
    let test_file = temp_dir.join("test_func.ln");
    fs::write(&test_file, src).unwrap();

    let exe_path = driver.build(&test_file, None).expect("Build should succeed");
    let output = std::process::Command::new(&exe_path).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "49");

    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&exe_path);
}

#[test]
fn test_compile_pass_dynamic_reassignment() {
    let mut driver = Driver::new();
    let temp_dir = std::env::temp_dir().join("lingo_pass_tests");
    fs::create_dir_all(&temp_dir).unwrap();

    let src = "def main():\n    x = 10\n    x = \"hello\"\n    println(x)\n";
    let test_file = temp_dir.join("test_dynamic.ln");
    fs::write(&test_file, src).unwrap();

    let exe_path = driver.build(&test_file, None).expect("Build should succeed");
    let output = std::process::Command::new(&exe_path).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "hello");

    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&exe_path);
}

#[test]
fn test_compile_pass_map_and_math() {
    let mut driver = Driver::new();
    let temp_dir = std::env::temp_dir().join("lingo_pass_tests");
    fs::create_dir_all(&temp_dir).unwrap();

    let src = "def main():\n    m = {\"key\": 100}\n    m[\"key\"] = m[\"key\"] + 50\n    println(m[\"key\"])\n    println(math_abs(-25))\n    println(len(m))\n";
    let test_file = temp_dir.join("test_map_math.ln");
    fs::write(&test_file, src).unwrap();

    let exe_path = driver.build(&test_file, None).expect("Build should succeed");
    let output = std::process::Command::new(&exe_path).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().map(|l| l.trim()).collect();
    assert_eq!(lines, vec!["150", "25", "1"]);

    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&exe_path);
}
