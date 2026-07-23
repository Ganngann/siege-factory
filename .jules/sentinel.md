## 2024-05-24 - [Path Traversal in Mod Loading]
**Vulnerability:** The mod loading mechanism (`load_data`, `load_texture`, `load_story`) in `src/core/modding.rs` accepted unsanitized file paths (such as `../` and absolute paths like `/etc/passwd` or `C:\`) directly into `PathBuf::join()`.
**Learning:** Rust's `PathBuf::join()` has a highly dangerous behavior where if the argument is an absolute path (or starts with a root/prefix), it completely replaces the existing base path.
**Prevention:** Always validate user-provided paths dynamically by parsing them with `Path::components()` and explicitly rejecting `Component::ParentDir`, `Component::RootDir`, and `Component::Prefix` before passing them to `PathBuf::join()`.
