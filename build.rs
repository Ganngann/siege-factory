use std::path::Path;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=assets_src/");

    let src = Path::new("assets_src");
    let out = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("assets")
        .join("textures");
    fs::create_dir_all(&out).unwrap();

    process_dir(&src.join("buildings"), &out);
    process_dir(&src.join("units"), &out);
    process_dir(&src.join("items"), &out);

    generate_manifest(&out);
}

fn process_dir(dir: &Path, out: &Path) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().map_or(false, |e| e == "svg") {
            process_svg(&entry.path(), out);
        }
    }
}

fn process_svg(path: &Path, out: &Path) {
    let content = fs::read_to_string(path).unwrap();
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let (w, h) = extract_viewbox(&content);

    // Base layer: keep only <g id="base">
    if let Some(svg) = keep_group(&content, "base") {
        render_png(&svg, &out.join(format!("{}_base.png", stem)), w, h);
    }

    // Owner layer: keep only <g id="owner_color">
    if content.contains(r#"id="owner_color""#) {
        if let Some(svg) = keep_group(&content, "owner_color") {
            render_png(&svg, &out.join(format!("{}_owner.png", stem)), w, h);
        }
    }

    // Level layer: keep only <g id="level_color">
    if content.contains(r#"id="level_color""#) {
        if let Some(svg) = keep_group(&content, "level_color") {
            render_png(&svg, &out.join(format!("{}_level.png", stem)), w, h);
        }
    }
}

/// Keep only the <g id="..."> group (and <defs>), remove all other groups.
fn keep_group(svg: &str, id: &str) -> Option<String> {
    let start_tag = format!(r#"<g id="{}">"#, id);
    let group_start = svg.find(&start_tag)?;
    let after_start = &svg[group_start + start_tag.len()..];
    let group_end = after_start.find("</g>")?;
    let group = &svg[group_start..group_start + start_tag.len() + group_end + 4];

    let defs_start = svg.find("<defs>");
    let defs = defs_start.and_then(|ds| {
        let after = &svg[ds..];
        after.find("</defs>").map(|de| &svg[ds..ds + de + 7])
    });

    let vb = extract_viewbox_str(svg);

    let mut result = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" {} width="{}" height="{}">"#,
        vb.map(|s| format!(r#"viewBox="{}""#, s)).unwrap_or_default(),
        extract_viewbox(&svg).0,
        extract_viewbox(&svg).1,
    );
    if let Some(d) = defs {
        result.push_str(d);
    }
    result.push_str(group);
    result.push_str("</svg>");
    Some(result)
}

fn extract_viewbox(svg: &str) -> (u32, u32) {
    let re = regex::Regex::new(r#"viewBox="\d+\s+\d+\s+(\d+)\s+(\d+)""#).unwrap();
    if let Some(caps) = re.captures(svg) {
        let w: u32 = caps[1].parse().unwrap_or(64);
        let h: u32 = caps[2].parse().unwrap_or(64);
        (w, h)
    } else {
        (64, 64)
    }
}

fn extract_viewbox_str(svg: &str) -> Option<&str> {
    let re = regex::Regex::new(r#"viewBox="[^"]*""#).unwrap();
    re.find(svg).map(|m| {
        let s = m.as_str();
        &s[9..s.len() - 1] // strip viewBox=" and trailing "
    })
}

fn render_png(svg: &str, out_path: &Path, w: u32, h: u32) {
    let opt = resvg::usvg::Options::default();
    let tree = match resvg::usvg::Tree::from_data(svg.as_bytes(), &opt) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("usvg error for {}: {}", out_path.display(), e);
            return;
        }
    };

    let mut pixmap = match resvg::tiny_skia::Pixmap::new(w, h) {
        Some(p) => p,
        None => {
            eprintln!("tiny_skia: failed to create pixmap {}x{} for {}", w, h, out_path.display());
            return;
        }
    };

    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    if let Err(e) = pixmap.save_png(out_path) {
        eprintln!("Failed to save {}: {}", out_path.display(), e);
    }
}

fn generate_manifest(out: &Path) {
    let mut entries: Vec<String> = Vec::new();
    for entry in fs::read_dir(out).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().map_or(false, |e| e == "png") {
            entries.push(entry.path().file_stem().unwrap().to_str().unwrap().to_string());
        }
    }
    entries.sort();
    let content = format!("[\n{}\n]",
        entries.iter().map(|e| format!("  \"{}\"", e)).collect::<Vec<_>>().join(",\n"));
    fs::write(out.join("manifest.ron"), content).unwrap();
    eprintln!("Generated {} textures", entries.len());
}
