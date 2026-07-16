## 2024-07-16 - Mod Loading Path Traversal
**Vulnerability:** The ModRegistry loaded files blindly by joining an arbitrary string directly with the mod directory path (`am.path.join("data").join(filename)`), allowing path traversal (e.g., `../../../etc/passwd`).
**Learning:** This existed because file loading methods assumed incoming strings from mod definitions were intrinsically safe/well-formed.
**Prevention:** Always validate and sanitize strings intended to become file paths. Created `is_safe_filename` to block strings containing `..` or absolute path markers (`/`, `\`).
