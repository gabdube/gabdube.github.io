import { file_extension } from "./helpers";
import { GameInterface, GameStartParams } from "./game_interface";
import { EngineAssets } from "./assets";
import { Renderer } from "./renderer";
import { EngineWebSocket, WebSocketMessage } from "./websocket";
import { set_last_error } from "./error";

const UPDATE_MOUSE_POSITION = 0b001;
const UPDATE_MOUSE_BUTTONS  = 0b010;
const UPDATE_KEYS           = 0b100;

// Matches `MouseButton` in `game\src\inputs.rs`
const MOUSE_BUTTON_LEFT = 0;
const MOUSE_BUTTON_RIGHT = 1;
const MOUSE_BUTTON_CENTER = 2;

class GameInput {
    updates: number = 0;
    mouse_position: number[] = [0.0, 0.0];

    // true: button was pressed, false: button was released, null: button state wasn't changed
    left_mouse_button: boolean|null = null;    
    right_mouse_button: boolean|null = null;
    center_mouse_button: boolean|null = null;

    keys: Map<string, boolean> = new Map();
}

class Engine {
    ws: EngineWebSocket = new EngineWebSocket();

    game: GameInterface = new GameInterface();
    assets: EngineAssets = new EngineAssets();
    renderer: Renderer = new Renderer();
    input: GameInput = new GameInput();

    reload_client: boolean = false;
    reload: boolean = false;
    exit: boolean = false;
}

//
// Init
//

function init_handlers(engine: Engine) {
    const canvas = engine.renderer.canvas.element;
    const input_state = engine.input;

    canvas.addEventListener("mousemove", (event) => { 
        input_state.mouse_position[0] = event.clientX - canvas.offsetLeft;
        input_state.mouse_position[1] = event.clientY - canvas.offsetTop;
        input_state.updates |= UPDATE_MOUSE_POSITION;
    })

    canvas.addEventListener("mousedown", (event) => {
        input_state.updates |= UPDATE_MOUSE_BUTTONS;

        if (event.button === 0) { input_state.left_mouse_button = true; }
        else if (event.button === 1) { input_state.center_mouse_button = true; }
        else if (event.button === 2) { input_state.right_mouse_button = true; }
        
        event.preventDefault();
    })

    canvas.addEventListener("mouseup", (event) => {
        input_state.updates |= UPDATE_MOUSE_BUTTONS;

        if (event.button === 0) { input_state.left_mouse_button = false; }
        else if (event.button === 1) { input_state.center_mouse_button = false; }
        else if (event.button === 2) { input_state.right_mouse_button = false; }
        
        event.preventDefault();
    })

    canvas.addEventListener("contextmenu", (event) => { event.preventDefault(); });

    window.addEventListener("keydown", (event) => {
        input_state.keys.set(event.code, true);
        input_state.updates |= UPDATE_KEYS;
    });
    window.addEventListener("keyup", (event) => {
        // console.log(event.code);
        input_state.keys.set(event.code, false);
        input_state.updates |= UPDATE_KEYS;
    });
}

async function init(): Promise<Engine | null> {
    const app = new Engine();

    if (!app.renderer.init()) {
        return null;
    }

    let init_client = app.game.init();
    let init_assets = app.assets.init();
    let [client_ok, assets_ok] = await Promise.all([init_client, init_assets]);
    if (!client_ok || !assets_ok) {
        return null;
    }

    if (!app.renderer.init_default_resources(app.assets)) {
        return null;
    }
    
    const params: GameStartParams = { 
        max_texture_size: app.renderer.max_texture_size(),
        screen_width: app.renderer.canvas.width,
        screen_height: app.renderer.canvas.height,
    };
    if (!app.game.start(app.assets, params)) {
        return null;
    }

    init_handlers(app);

    app.ws.open();

    return app;
}

//
// Updates
//

function on_file_changed(engine: Engine, message: WebSocketMessage) {
    // Reloading is async so we don't execute it right away in the game loop.
    // See the `reload` function in this file
    const ext = file_extension(message.data);
    switch (ext) {
        case "wasm": {
            engine.reload_client = true;
            engine.reload = true;
            break;
        }
    }
}

/// Handle the updates received from the development server
function websocket_messages(engine: Engine) {
    const ws = engine.ws;
    if (!ws.open) {
        // We're using a static client with no dev server
        return;
    }

    for (let i=0; i<ws.messages_count; i++) {
        let message = ws.messages[i];
        switch (message.name) {
            case "FILE_CHANGED": {
                on_file_changed(engine, message);
                break;
            }
            default: {
                console.log("Unknown message:", message);
            }
        }
    }

    ws.messages_count = 0;
}

/// Check if the canvas size changed since the last call, and if so run the on resize logic
function handle_resize(engine: Engine) {
    if (engine.renderer.handle_resize()) {
        const width = engine.renderer.canvas.width;
        const height = engine.renderer.canvas.height;
        engine.game.resize(width, height)
    }
}

function game_input_updates(engine: Engine) {
    const inputs = engine.input;
    const game = engine.game.instance;

    if ((inputs.updates & UPDATE_MOUSE_POSITION) > 0) {
        game.update_mouse_position(inputs.mouse_position[0], inputs.mouse_position[1]);
    }

    if ((inputs.updates & UPDATE_MOUSE_BUTTONS) > 0) {
        if (inputs.left_mouse_button !== null) {
            game.update_mouse_buttons(MOUSE_BUTTON_LEFT, inputs.left_mouse_button);
        }

        if (inputs.right_mouse_button !== null) {
            game.update_mouse_buttons(MOUSE_BUTTON_RIGHT, inputs.right_mouse_button);
        }

        if (inputs.center_mouse_button !== null) {
            game.update_mouse_buttons(MOUSE_BUTTON_CENTER, inputs.center_mouse_button);
        }

        inputs.left_mouse_button = null;
        inputs.right_mouse_button = null;
        inputs.center_mouse_button = null;
    }

    if ((inputs.updates & UPDATE_KEYS) > 0) {
        for (let entry of inputs.keys.entries()) {
            game.update_keys(entry[0], entry[1]);
        }
    }

    inputs.keys.clear();
    inputs.updates = 0;
}

/// Execute the game logic of the client for the current frame
function game_updates(engine: Engine, time: DOMHighResTimeStamp) {
    game_input_updates(engine)
    engine.game.instance.update(time)
}

/// Reads the rendering updates generated by the game client
function renderer_updates(engine: Engine) {
    engine.renderer.update(engine.game);
}

function update(engine: Engine, time: DOMHighResTimeStamp) {
    websocket_messages(engine);
    handle_resize(engine);
    game_updates(engine, time);
    renderer_updates(engine);
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
    const engine = await init();
    if (!engine) {
        console.log("Failed to initialize application");
        return;
    }

    boundedRun = run.bind(null, engine);
    boundedRun();
}

init_app();
