@echo off
echo ====================================
echo   Running Lingo 1.0 Full Test Suite
echo ====================================
echo.

echo --- Running examples/hello.ln ---
cargo run --quiet -- run examples/hello.ln
echo.

echo --- Running examples/functions_and_types.ln ---
cargo run --quiet -- run examples/functions_and_types.ln
echo.

echo --- Running examples/control_flow.ln ---
cargo run --quiet -- run examples/control_flow.ln
echo.

echo --- Running examples/concurrency.ln ---
cargo run --quiet -- run examples/concurrency.ln
echo.

echo --- Running examples/ffi_interop.ln ---
cargo run --quiet -- run examples/ffi_interop.ln
echo.

echo --- Running examples/stdlib_test.ln ---
cargo run --quiet -- run examples/stdlib_test.ln
echo.

echo --- Running examples/data_processor.ln ---
cargo run --quiet -- run examples/data_processor.ln
echo.

echo --- Running examples/map_and_math.ln ---
cargo run --quiet -- run examples/map_and_math.ln
echo.

echo --- Running bootstrap/min_compiler.ln ---
cargo run --quiet -- run bootstrap/min_compiler.ln
echo.

echo All Lingo 1.0 test suites compiled and executed successfully!
