use lingo_driver::Driver;
use std::fs;

#[test]
fn test_compile_fail_syntax_error() {
    let mut driver = Driver::new();
    let temp_dir = std::env::temp_dir().join("lingo_fail_tests");
    fs::create_dir_all(&temp_dir).unwrap();

    let src = "def main(:\n    println(1)\n";
    let test_file = temp_dir.join("test_syntax_fail.ln");
    fs::write(&test_file, src).unwrap();

    let res = driver.build(&test_file, None);
    assert!(res.is_err());
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_compile_fail_undeclared_name() {
    let mut driver = Driver::new();
    let temp_dir = std::env::temp_dir().join("lingo_fail_tests");
    fs::create_dir_all(&temp_dir).unwrap();

    let src = "def main():\n    println(unknown_symbol_xyz)\n";
    let test_file = temp_dir.join("test_undeclared_fail.ln");
    fs::write(&test_file, src).unwrap();

    let res = driver.build(&test_file, None);
    assert!(res.is_err());
    let _ = fs::remove_file(&test_file);
}
