use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn validate_manifest(m: &Value) -> Result<(), String> {
    if m["manifest_version"] != 3
        || m["action"]["default_popup"] != "popup.html"
        || m.get("options_ui").is_some()
        || m.get("externally_connectable").is_some()
    {
        return Err("Only the MV3 popup interface is allowed".into());
    }
    if m["permissions"] != json!(["activeTab", "storage"])
        || m["host_permissions"] != json!(["https://api.fxtwitter.com/*"])
    {
        return Err("Unexpected browser permissions".into());
    }
    let matches = json!([
        "https://x.com/*",
        "https://www.x.com/*",
        "https://twitter.com/*",
        "https://www.twitter.com/*"
    ]);
    if m["content_scripts"][0]["matches"] != matches
        || m["content_scripts"][0]["js"] != json!(["adapters/content.js"])
    {
        return Err("Unexpected content-script scope".into());
    }
    if m["web_accessible_resources"] != json!([{"resources":["styles/app.css"],"matches":matches}])
    {
        return Err("Only the stylesheet may be exposed to X".into());
    }
    let csp = m["content_security_policy"]["extension_pages"]
        .as_str()
        .ok_or("Missing CSP")?;
    if !csp.contains("script-src 'self' 'wasm-unsafe-eval';")
        || csp.contains("'unsafe-eval'")
        || csp.contains("'unsafe-inline'")
    {
        return Err("Unexpected executable policy".into());
    }
    Ok(())
}
fn collect(source: &Path, name: &str, out: &mut Vec<(PathBuf, String)>) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source).map_err(|e| e.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("Symlinks are not permitted in release archives".into());
    }
    if metadata.is_dir() {
        let mut entries = fs::read_dir(source)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let child = entry.file_name().to_string_lossy().into_owned();
            if matches!(child.as_str(), "artifacts" | "__pycache__") {
                continue;
            }
            let next = if name.is_empty() {
                child
            } else {
                format!("{name}/{child}")
            };
            collect(&entry.path(), &next, out)?;
        }
    } else {
        if name.starts_with('/') || name.split('/').any(|part| part == "..") {
            return Err("Unsafe archive entry".into());
        }
        out.push((source.into(), name.into()));
    }
    Ok(())
}
pub fn validate(root: &Path) -> Result<(), String> {
    let m: Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    validate_manifest(&m)?;
    for name in [
        "popup.html",
        "adapters/background.js",
        "adapters/content.js",
        "adapters/ui.js",
        "styles/app.css",
        "brand/mark.svg",
        "brand/icon-16.png",
        "brand/icon-32.png",
        "brand/icon-48.png",
        "brand/icon-128.png",
    ] {
        if !root.join(name).is_file() {
            return Err(format!("Missing packaged resource: {name}"));
        }
    }
    let mut entries = vec![];
    collect(root, "", &mut entries)?;
    if entries
        .iter()
        .filter(|(_, name)| name.ends_with(".html"))
        .map(|(_, name)| name.as_str())
        .collect::<Vec<_>>()
        != vec!["popup.html"]
    {
        return Err("The popup must be the only HTML interface".into());
    }
    for module in ["runtime", "ui"] {
        let bytes = fs::read(root.join(format!("pkg/{module}/limitless_{module}_bg.wasm")))
            .map_err(|e| e.to_string())?;
        if !bytes.starts_with(b"\0asm") {
            return Err("Invalid packaged WASM".into());
        }
    }
    Ok(())
}
pub fn archive(destination: &Path, sources: &[(&Path, &str)]) -> Result<(), String> {
    let mut entries = vec![];
    for (source, name) in sources {
        collect(source, name, &mut entries)?;
    }
    entries.sort_by(|a, b| a.1.cmp(&b.1));
    let file = fs::File::create(destination).map_err(|e| e.to_string())?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for (source, name) in entries {
        writer
            .start_file(name, options)
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&fs::read(source).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
    writer.finish().map_err(|e| e.to_string())?;
    Ok(())
}
pub fn release() -> Result<(), String> {
    fs::create_dir_all("dist/releases").map_err(|e| e.to_string())?;
    for browser in ["chromium", "firefox"] {
        let root = PathBuf::from(format!("dist/{browser}"));
        validate(&root)?;
        archive(
            Path::new(&format!(
                "dist/releases/limitlessx-{browser}-{}.zip",
                env!("CARGO_PKG_VERSION")
            )),
            &[(&root, "")],
        )?;
    }
    let paths = [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo",
        "crates",
        "extension",
        "scripts",
        "docs",
        "README.md",
        "LICENSE",
        "tests",
        ".github",
    ];
    let sources: Vec<_> = paths.iter().map(|path| (Path::new(path), *path)).collect();
    archive(
        Path::new(&format!(
            "dist/releases/limitlessx-source-{}.zip",
            env!("CARGO_PKG_VERSION")
        )),
        &sources,
    )?;
    println!("Validated popup-only browser packages and source archive in dist/releases");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_broad_permissions_and_extra_extension_pages() {
        let original: Value =
            serde_json::from_str(include_str!("../../../extension/manifests/chromium.json"))
                .unwrap();
        assert!(validate_manifest(&original).is_ok());
        for (key, value) in [
            (
                "permissions",
                serde_json::json!(["activeTab", "storage", "history"]),
            ),
            ("host_permissions", serde_json::json!(["<all_urls>"])),
            ("options_ui", serde_json::json!({"page":"options.html"})),
            (
                "web_accessible_resources",
                serde_json::json!([{"resources":["pkg/runtime/*"],"matches":["<all_urls>"]}]),
            ),
        ] {
            let mut m = original.clone();
            m[key] = value;
            assert!(validate_manifest(&m).is_err(), "accepted unsafe {key}");
        }
    }
    #[test]
    fn archives_are_repeatable_and_have_relative_entry_names() {
        let root =
            std::env::temp_dir().join(format!("limitlessx-archive-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("source")).unwrap();
        std::fs::write(root.join("source/a.txt"), "hello").unwrap();
        let source = root.join("source");
        let a = root.join("a.zip");
        let b = root.join("b.zip");
        archive(&a, &[(&source, "extension")]).unwrap();
        archive(&b, &[(&source, "extension")]).unwrap();
        assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
        let mut z = zip::ZipArchive::new(std::fs::File::open(a).unwrap()).unwrap();
        assert_eq!(z.by_index(0).unwrap().name(), "extension/a.txt");
        std::fs::remove_dir_all(root).unwrap();
    }
}
