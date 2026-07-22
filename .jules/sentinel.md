## 2025-02-14 - Prevent Path Traversal in Mod Loading
**Vulnerability:** Path traversal (arbitrary file read) via `modding.rs`. Functions like `load_data` appended unvalidated input to a base path. Using absolute paths (which overwrite the base path) or `../` allowed reading outside the intended directories.
**Learning:** `PathBuf::join` replaces the entire path if given an absolute path, and traverses directories if given `../`. Since filenames here come from mod data or user input, they must be validated.
**Prevention:** Filter all user-controlled path inputs before using `PathBuf::join` by ensuring `Path::components().any(...)` is false for `Component::ParentDir`, `Component::RootDir`, and `Component::Prefix(_)`.
