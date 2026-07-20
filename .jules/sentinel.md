## 2025-02-18 - Prevent Path Traversal in Mod Loading
**Vulnerability:** Path traversal (e.g., `../`, `/`, `C:\`) in `ModRegistry` data loading functions (`load_data`, `load_all_data`, `load_story`), which build paths dynamically with `PathBuf::join(filename)`.
**Learning:** `PathBuf::join()` replaces the entire path if the appended component is an absolute path. Any unsanitized string used with it can result in directory traversal or absolute path overwrites.
**Prevention:** Sanitize mod-supplied or dynamically generated filenames by checking `Path::components()` and rejecting `ParentDir`, `RootDir`, and `Prefix` components before passing them to `join()`.
