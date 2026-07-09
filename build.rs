use std::path::Path;

fn main() {
    let ttf_path = Path::new("assets").join("fonts").join("font.ttf");
    if ttf_path.exists() {
        let size = std::fs::metadata(&ttf_path).map(|m| m.len()).unwrap_or(0);
        println!("cargo:warning=Font ready: {} ({} bytes)", ttf_path.display(), size);
    } else {
        println!("cargo:warning=assets/fonts/font.ttf not found — place a .ttf file there for proper font rendering");
    }
}
