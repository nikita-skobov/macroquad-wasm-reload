# macroquad-wasm-reload

This project runs a local webserver that serves a simple html page that will
use websockets to repeatedly ping the webserver to check for updates to a macroquad project.
if an update is found, the frontend hot-reloads the wasm directly in the browser.

# How to use:

```sh
cargo run
```

It will ask you for a path to your project. This should be some other directory
that has your macroquad project. It will continually check for updates to that directory, and if any file was updated, it will re-build the wasm.

After providing the path to a project, open your browser to http://localhost:3030


