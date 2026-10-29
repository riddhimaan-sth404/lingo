# Lingo Cross-Platform Portability (Milestone 31)

## Supported Targets
1. **Windows**: `x86_64-pc-windows-msvc`
   - Object format: COFF (`.obj`)
   - Linker: `lld-link` / MSVC `link.exe`
   - Platform libraries: `ntdll.lib`, `kernel32.lib`, `userenv.lib`, `ws2_32.lib`, `advapi32.lib`, `bcrypt.lib`
2. **Linux**: `x86_64-unknown-linux-gnu`
   - Object format: ELF (`.o`)
   - Linker: `ld.lld` / `gcc`
   - Platform libraries: `libpthread.so`, `libdl.so`, `libc.so`, `libm.so`

## Platform Abstraction
The compiler driver detects the host operating system dynamically and automatically configures target triples and linking flags accordingly.
