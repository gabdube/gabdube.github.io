# Third Codex

This is the source code of my personal blog.

https://gabdube.github.io/

## Useful commands

Building a demo from source (replace "retained_gui" by the right folder)

```
cd articles/retained_gui/client_src 
wasm-pack build --out-dir ../build --target web
cp ../build/retained_gui.js ../release/
cp ../build/retained_gui_bg.wasm ../release/
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

## License

All code in this project use the MIT License unless specified otherwise.