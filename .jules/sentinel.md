## 2025-07-19 - Path Traversal in Mod Loading
**Vulnerability:** The mod loader accepted arbitrary filenames to `load_data()`, `load_texture()`, etc., joining them with the mod's base path using `PathBuf::join()`. If a maliciously crafted mod passed a filename like `../../../etc/passwd` or `/etc/passwd`, it could read arbitrary system files. Joining an absolute path in Rust completely replaces the base path.
**Learning:** Functions accepting dynamic filenames (from configs, mods, or user input) to build file paths must sanitize the input against both parent directory traversal and absolute path replacement.
**Prevention:** Always validate path components. Use `Path::components().any(...)` and reject `ParentDir`, `RootDir`, and `Prefix` before appending to a trusted base path.
