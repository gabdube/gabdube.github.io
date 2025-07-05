# Third Codex

This is the source code of my personal blog.

https://gabdube.github.io/

## Useful commands

Building the rust source of a demo

```
cd articles/minimal_retained_rust_gui/wasm_src
wasm-pack build --out-dir ../build --target web
cp ../build/minimal_retained_rust_gui_demo.js ../
cp ../build/minimal_retained_rust_gui_demo_bg.wasm ../

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


## Tools

The tools folder includes a rust program that can preprocess assets. Unprocessed assets are not included with this project, however the `tools`
utility is still included for completeness sake.

```
cargo run --release -p tools -- [command_name] *command_args*
```

