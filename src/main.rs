use std::{io::{BufRead, Read}, path::PathBuf, process::Command};

use warp::{Filter, hyper::Response};

const INDEX_HTML: &str = include_str!(env!("INDEX_FILE"));
const JS_FILE: &str = include_str!(env!("JS_FILE"));
const CSS_FILE: &str = include_str!(env!("CSS_FILE"));

/// returns path to .wasm file
fn build_wasm(cargo_path: &PathBuf) -> PathBuf {
    println!("Building wasm in {:?}", cargo_path);
    let out = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .current_dir(cargo_path)
        .output().expect(&format!("Failed to run cargo build in {:?}", cargo_path));

    if !out.status.success() {
        let err_str = String::from_utf8_lossy(&out.stderr);
        panic!("{err_str}");
    }
    let base_name = cargo_path.file_name().expect("Failed to get final name of cargo path");
    let wasm_name = format!("{}.wasm", base_name.to_string_lossy().to_string());
    let mut out_file = cargo_path.clone();
    out_file.push("target");
    out_file.push("wasm32-unknown-unknown");
    out_file.push("debug");
    out_file.push(wasm_name);
    if !out_file.exists() {
        panic!("Failed to find .wasm file after build {:?}", out_file);
    }
    println!("Got wasm {:?}", out_file);
    out_file

}

#[tokio::main]
async fn main() {
    let cargo_dir: PathBuf;
    loop {
        println!("Enter a path to watch and 'cargo build' each time it changes:");
        let mut stdinh = std::io::stdin().lock();
        let mut line = String::new();
        stdinh.read_line(&mut line).expect("Failed to read line");
        line = line.trim().to_string();
        println!("Looking up cargo package {line}");
        cargo_dir = PathBuf::from(&line);
        if !cargo_dir.exists() {
            println!("That doesnt exist, try again");
        }
        break;
    }
    let wasm_file = build_wasm(&cargo_dir);
    let index_route: _ = warp::path("index.html")
        .or(warp::path::end())
        .map(|_| warp::reply::html(INDEX_HTML));
    let js_route: _ = warp::path("assets").and(warp::path(env!("JS_FILE_BASE")))
        .map(|| {
            Response::builder()
            .header("content-type", "application/javascript")
            .body(JS_FILE)
        });
    let css_route: _ = warp::path("assets").and(warp::path(env!("CSS_FILE_BASE")))
        .map(|| warp::reply::html(CSS_FILE));
    let wasm_route: _ = warp::path("current.wasm")
        .map(move || {
            let wasm_file = build_wasm(&cargo_dir);
            let mut data = std::fs::File::open(&wasm_file).expect("Failed to read wasm file");
            let mut bytes = vec![];
            data.read_to_end(&mut bytes).expect("Failed to read wasm file");
            Response::builder()
                .header("content-type", "application/wasm")
                .body(bytes)
        });
    let routes: _ = warp::any().and(
        index_route
            .or(js_route)
            .or(css_route)
            .or(wasm_route)
    );

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
