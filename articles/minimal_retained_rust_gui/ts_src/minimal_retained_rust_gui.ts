import { GameInterface, GameInterfaceStartupParams } from "./game_interface";
import { EngineAssets } from "./assets";
import { Renderer } from "./renderer";
import { EngineWebSocket, WebSocketMessage } from "./websocket";
import { set_last_error } from "./error";

class Engine {
    ws: EngineWebSocket = new EngineWebSocket();

    game: GameInterface = new GameInterface();
    assets: EngineAssets = new EngineAssets();
    renderer: Renderer = new Renderer();

    reload_client: boolean = false;
    reload: boolean = false;
    exit: boolean = false;
}

//
// Init
//

function init_handlers(engine: Engine) {
}

function start_client(engine: Engine): boolean {
    const params: GameInterfaceStartupParams = { 
        max_texture_size: engine.renderer.max_texture_size(),
        screen_width: engine.renderer.canvas.width,
        screen_height: engine.renderer.canvas.height,
    };

    return engine.game.start(engine.assets, params);
}

async function init(): Promise<Engine | null> {
    const engine = new Engine();

    if (!engine.renderer.init()) {
        return null;
    }

    let init_client = engine.game.init();
    let init_assets = engine.assets.init();
    let [client_ok, assets_ok] = await Promise.all([init_client, init_assets]);
    if (!client_ok || !assets_ok) {
        return null;
    }

    if (!engine.renderer.init_default_resources(engine.assets)) {
        return null;
    }
 
    if (!start_client(engine)) {
        return null;
    }

    init_handlers(engine);

    engine.ws.open();

    (window as any).engine = engine;

    return engine;
}

//
// Updates
//

function update(engine: Engine, time: DOMHighResTimeStamp) {
}

//
// Render
//

function render(engine: Engine) {
    engine.renderer.render();
}

//
// Reload
//

async function reload(engine: Engine) {
    if (engine.reload_client) {
        const reloaded = await engine.game.reload();
        if (!reloaded) {
            set_last_error("Failed to reload wasm module");
            engine.exit = true;
        }
    }

    engine.reload = false;
}

//
// Runtime
//

let boundedRun = () => {};

function run(engine: Engine) {
    if (engine.exit) {
        return;
    }

    update(engine, performance.now());
    render(engine);

    if (engine.reload) {
        reload(engine)
            .then(() => requestAnimationFrame(boundedRun) );
    } else {
        requestAnimationFrame(boundedRun);
    }
}

async function init_app() {
    const demo = document.getElementById("demo") as HTMLCanvasElement;
    if (demo.clientWidth == 0 || demo.clientHeight == 0) {
        return;
    }

    const engine = await init();
    if (!engine) {
        console.log("Failed to initialize application");
        return;
    }

    boundedRun = run.bind(null, engine);
    boundedRun();

    return;
}

init_app();
