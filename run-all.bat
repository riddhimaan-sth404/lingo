@echo off
echo ====================================
echo   Running Lingo Examples
echo ====================================
echo.

echo --- Running examples/hello.ln ---
cargo run --quiet -- run examples/hello.ln
echo.

echo --- Running examples/rust_crates_and_types.ln ---
cargo run --quiet -- run examples/rust_crates_and_types.ln
echo.

echo --- Running examples/crates_interop.ln ---
cargo run --quiet -- run examples/crates_interop.ln
echo.

echo All examples executed successfully!
