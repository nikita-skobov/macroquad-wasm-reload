use std::{process::Command, path::PathBuf};

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=web/");

    let home = std::env::var("HOME").expect("Failed to find HOME env var");
    let path = std::env::var("PATH").expect("Failed to find PATH var");
    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to find cargo manifest dir");
    let dir = PathBuf::from(dir);
    let mut web_dir = dir.clone();
    web_dir.push("web");
    if !web_dir.exists() {
        panic!("Failed to find web/ directory. make sure you are building this from mquad-wasm-reload root");
    }
    let mut node_modules_dir = web_dir.clone();
    node_modules_dir.push("node_modules");
    if !node_modules_dir.exists() {
        println!("npm install...");
        let cmd = Command::new("npm")
            .current_dir(&web_dir)
            .env("PATH", &path)
            .arg("install").output()
            .expect("Failed to run npm install");
        if !cmd.status.success() {
            let err_str = String::from_utf8_lossy(&cmd.stderr);
            panic!("{err_str}");
        }
        println!("Successfully ran npm install");
    }

    // build frontend into dist/
    // TODO: bug in rust-analyzer it doesnt set PATH correctly, so currently
    // we manually have to tell it where node is. the problem is this will be different for
    // different users. need to fix rust-analyzer to have it set path correctly.
    let path = format!("{path}:{home}/.nvm/versions/node/v18.13.0/bin");
    let cmd = Command::new("npm")
        .current_dir(&web_dir)
        .env("PATH", path)
        .arg("run").arg("build")
        .output().expect("Failed to run npm run build");
    if !cmd.status.success() {
        let err_str = String::from_utf8_lossy(&cmd.stderr);
        panic!("{err_str}");
    }
    let mut dist_dir = web_dir.clone();
    dist_dir.push("dist");
    let mut index_file = dist_dir.clone();
    index_file.push("index.html");
    let index_file_full = index_file.canonicalize().expect("Failed to canonicalize index.html");

    println!("Successfully ran npm run build");
    println!("cargo:rustc-env=INDEX_FILE={}", index_file_full.to_string_lossy().to_string());

    let mut asset_dir = dist_dir.clone();
    asset_dir.push("assets");
    for file in std::fs::read_dir(&asset_dir).expect("Failed to read asset dir") {
        let fileentry = file.expect("Failed to read asset dir entry");
        let fpath = fileentry.path();
        let fpath = fpath.canonicalize().expect("Failed to canonicalize file path");
        let ext = fpath.extension().expect("Failed to get extension");
        let ext = ext.to_string_lossy().to_string();
        let fpathbase = fpath.file_name().expect("Failed to get file path base name");
        if ext == "js" {
            println!("cargo:rustc-env=JS_FILE={}", fpath.to_string_lossy().to_string());
            println!("cargo:rustc-env=JS_FILE_BASE={}", fpathbase.to_string_lossy().to_string());
        } else if ext == "css" {
            println!("cargo:rustc-env=CSS_FILE={}", fpath.to_string_lossy().to_string());
            println!("cargo:rustc-env=CSS_FILE_BASE={}", fpathbase.to_string_lossy().to_string());
        }
    }
}
