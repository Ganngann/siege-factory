## 2024-07-24 - Path Traversal Vulnerability in Modding File Loader
**Vulnerability:** Path traversal and absolute path overwrite vulnerabilities existed in `src/core/modding.rs` where user-provided filenames were passed directly into `PathBuf::join()` to construct load paths.
**Learning:** `PathBuf::join` in Rust completely replaces the base path if the argument is an absolute path. This allowed potential arbitrary file reading on the user's filesystem (e.g., passing `/etc/passwd` to `load_data` would load `/etc/passwd` instead of `mods/data/etc/passwd`).
**Prevention:** Always validate dynamically constructed file paths that incorporate user or mod input by rejecting `ParentDir`, `RootDir`, and `Prefix` path components before calling `join`.
