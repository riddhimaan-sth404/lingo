# Lingo Module System

## Modules and Source Files
- Each `.ln` source file constitutes a module.
- Modules expose top-level function and class declarations.
- Import statements load module exports:
  ```python
  import math
  from utils import helper_func
  ```
- Circular imports are rejected at compile time during module graph resolution.
