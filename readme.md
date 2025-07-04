# Third Codex

This is the source code of my personal blog.

https://gabdube.github.io/

## Useful commands

Building the rust source of a demo

```
cd articles/minimal_retained_rust_gui/wasm_src
wasm-pack build --out-dir ../build --target web
cp ../build/minimal_retained_rust_gui.js ../
cp ../build/minimal_retained_rust_gui_bg.wasm ../

```

Compiling the typescript source of a demo

```
npm install
npx rollup --config rollup.config.mjs --watch
```

Starting the local server

```
cargo run --release -p local-server
```

