use std::{io::{BufRead, Read}, path::PathBuf, process::Command, collections::HashMap};

use warp::{Filter, hyper::Response, ws::{WebSocket, Message}};
use futures_util::{StreamExt, SinkExt};


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

#[derive(Debug)]
pub struct State {
    pub hashes: HashMap<String, u32>,
}

pub fn dir_has_changes(p: &PathBuf, state: &mut State) -> Result<bool, String> {
    let readdir = p.read_dir().map_err(|e| e.to_string())?;
    for file in readdir {
        let file = file.map_err(|e| e.to_string())?;
        let path = file.path();
        if path.is_dir() {
            if path.ends_with("target") || path.ends_with(".git") {
                // skip these
                continue;
            }
            if dir_has_changes(&path, state)? {
                return Ok(true);
            }
        } else {
            let path_str = path.to_string_lossy().to_string();
            if !state.hashes.contains_key(&path_str) {
                // println!("Has changes NOT_IN_MAP {}", path_str);
                let mut val = std::fs::File::open(&path_str).map_err(|e| e.to_string())?;
                let mut data = vec![];
                val.read_to_end(&mut data).map_err(|e| e.to_string())?;
                let c = std::io::Cursor::new(data);
                let current = adler32::adler32(c).unwrap_or_default();
                state.hashes.insert(path_str.clone(), current);
                // state.hashes.insert(path_str, 0);
                return Ok(true);
            }
            let prev = state.hashes[&path_str];
            let mut val = std::fs::File::open(&path_str).map_err(|e| e.to_string())?;
            let mut data = vec![];
            val.read_to_end(&mut data).map_err(|e| e.to_string())?;
            let c = std::io::Cursor::new(data);
            let current = adler32::adler32(c).unwrap_or_default();
            state.hashes.insert(path_str.clone(), current);
            if prev != current {
                // println!("Has changes HASH_NOT_MATCH {}", &path_str);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn initial_state(p: &PathBuf, state: &mut State) -> Result<bool, String> {
    let readdir = p.read_dir().map_err(|e| e.to_string())?;
    for file in readdir {
        let file = file.map_err(|e| e.to_string())?;
        let path = file.path();
        if path.is_dir() {
            if path.ends_with("target") || path.ends_with(".git") {
                // skip these
                continue;
            }
            if initial_state(&path, state)? {
                // return Ok(true);
            }
        } else {
            let path_str = path.to_string_lossy().to_string();
            let mut val = std::fs::File::open(&path_str).map_err(|e| e.to_string())?;
            let mut data = vec![];
            val.read_to_end(&mut data).map_err(|e| e.to_string())?;
            let c = std::io::Cursor::new(data);
            let current = adler32::adler32(c).expect("Failed to get adler!");
            state.hashes.insert(path_str.clone(), current);
        }
    }
    Ok(false)
}

pub static mut HAS_CHANGES: bool = false;


async fn user_connected(ws: WebSocket) {
    println!("USER CONNECTED");
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(_msg) => {
                let is_changed = unsafe {
                    HAS_CHANGES
                };
                if is_changed {
                    "true"
                } else {
                    "false"
                }
            },
            Err(_) => {
                break;
            }
        };
        let m = Message::text(msg);
        let _ = user_ws_tx.send(m).await;
    }
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
    let _ = build_wasm(&cargo_dir);

    let cargo_dir_copy = cargo_dir.clone();
    std::thread::spawn(move || {
        let mut s = State { hashes: Default::default() };
        let _ = initial_state(&cargo_dir_copy, &mut s).expect("Failed to get initial state");
        println!("{:#?}", s);
        loop {
            match dir_has_changes(&cargo_dir_copy, &mut s) {
                Ok(a) => {
                    if a {
                        println!("HAS CHANGES");
                        unsafe {
                            HAS_CHANGES = true;
                        }
                    }
                },
                Err(e) => {
                    println!("Err checking if has changes {}", e);
                },
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });

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
        .map(|| {
            Response::builder()
            .header("content-type", "text/css")
            .body(CSS_FILE)
        });
    let wasm_route: _ = warp::path("current.wasm")
        .map(move || {
            let wasm_file = build_wasm(&cargo_dir);
            let mut data = std::fs::File::open(&wasm_file).expect("Failed to read wasm file");
            let mut bytes = vec![];
            data.read_to_end(&mut bytes).expect("Failed to read wasm file");
            unsafe {
                HAS_CHANGES = false;
            }
            Response::builder()
                .header("content-type", "application/wasm")
                .body(bytes)
        });
    let wsroute: _ = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(move |socket| user_connected(socket))
        });
    let routes: _ = warp::any().and(
        index_route
            .or(js_route)
            .or(css_route)
            .or(wasm_route)
            .or(wsroute)
    );

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
