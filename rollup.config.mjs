import typescript from '@rollup/plugin-typescript';

const plugins = [typescript({
    compilerOptions: {
        target: "ES2020",
    }
})];

export default [
    {
        input: './articles/navmesh_pathfinding/ts_src/navmesh_pathfinding.ts',
        output: { file: './articles/navmesh_pathfinding/navmesh_pathfinding.js', format: 'es' },
        plugins
    },
    {
        input: './articles/minimal_retained_rust_gui/ts_src/minimal_retained_rust_gui.ts',
        output: { file: './articles/minimal_retained_rust_gui/minimal_retained_rust_gui.js', format: 'es' },
        plugins
    },
];
