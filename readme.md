# Third Codex

This is the source code of my personal blog.

## Useful commands

Building the rust source of a demo

```
cd articles/navmesh_pathfinding/wasm_src
wasm-pack build --out-dir ../build --target web
cp ../build/navmesh_pathfinding_demo.js ../
cp ../build/navmesh_pathfinding_demo_bg.wasm ../

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

