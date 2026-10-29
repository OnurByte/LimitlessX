mod package;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
fn run(cmd: &mut Command) -> Result<(), String> {
    let status = cmd.status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {cmd:?}"))
    }
}
fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for e in fs::read_dir(from).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        if e.path().is_dir() {
            copy_dir(&e.path(), &to.join(e.file_name()))?
        } else {
            fs::copy(e.path(), to.join(e.file_name())).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
fn build(browser: &str) -> Result<(), String> {
    if !["chromium", "firefox"].contains(&browser) {
        return Err("Expected chromium or firefox".into());
    }
    run(Command::new("cargo").args([
        "build",
        "--locked",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        "limitless-runtime",
        "-p",
        "limitless-ui",
    ]))?;
    let out = PathBuf::from("dist").join(browser);
    if out.exists() {
        fs::remove_dir_all(&out).map_err(|e| e.to_string())?;
    }
    copy_dir(Path::new("extension/static"), &out)?;
    copy_dir(Path::new("extension/adapters"), &out.join("adapters"))?;
    fs::copy(
        format!("extension/manifests/{browser}.json"),
        out.join("manifest.json"),
    )
    .map_err(|e| e.to_string())?;
    for name in ["runtime", "ui"] {
        let dest = out.join("pkg").join(name);
        fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        run(
            Command::new(env::var("WASM_BINDGEN").unwrap_or("wasm-bindgen".into()))
                .arg(format!(
                    "target/wasm32-unknown-unknown/release/limitless_{name}.wasm"
                ))
                .args(["--target", "web", "--no-typescript", "--out-dir"])
                .arg(&dest),
        )?;
        let wasm = dest.join(format!("limitless_{name}_bg.wasm"));
        run(
            Command::new(env::var("WASM_OPT").unwrap_or("wasm-opt".into()))
                .arg(&wasm)
                .args([
                    "-Os",
                    "--enable-bulk-memory",
                    "--enable-nontrapping-float-to-int",
                    "-o",
                ])
                .arg(&wasm),
        )?;
    }
    package::validate(&out)?;
    println!("Built {}", out.display());
    Ok(())
}
fn main() -> ExitCode {
    env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap(),
    )
    .unwrap();
    let args: Vec<_> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("build") => build(args.get(1).map(String::as_str).unwrap_or("")),
        Some("package") => (|| {
            build("chromium")?;
            build("firefox")?;
            package::release()
        })(),
        Some("test") => run(Command::new("cargo").args(["test", "--workspace", "--locked"])),
        _ => Err("Usage: cargo xtask build <chromium|firefox> | test | package".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
