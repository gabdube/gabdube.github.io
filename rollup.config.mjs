import typescript from '@rollup/plugin-typescript';

const plugins = [typescript({
    compilerOptions: {
        target: "es2021",
    }
})];

export default [
    {
        input: './articles/navmesh_pathfinding/ts_src/navmesh_pathfinding.ts',
        output: { file: './articles/navmesh_pathfinding/release/navmesh_pathfinding.js', format: 'es' },
        plugins
    },
    {
        input: './articles/retained_gui/engine_src/retained_gui.ts',
        output: { file: './articles/retained_gui/release/engine.js', format: 'es' },
        plugins
    },
];
