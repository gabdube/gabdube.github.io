import { EngineAssets } from "./assets";
import { init_codeview } from "./codeview";
import { set_last_error } from "./error";
import { GameInterface, GameInterfaceStartupParams } from "./game_interface";
import { file_extension } from "./helpers";
import { Renderer } from "./renderer";
import { EngineWebSocket, WebSocketMessage } from "./websocket";


const UPDATE_MOUSE_POSITION = 0b0001;
const UPDATE_MOUSE_BUTTONS  = 0b0010;
const UPDATE_KEYS           = 0b0100;
const UPDATE_WHEEL          = 0b1000;

// Matches `MouseButton` in `game\src\inputs.rs`
const MOUSE_BUTTON_LEFT = 0;
const MOUSE_BUTTON_RIGHT = 1;
const MOUSE_BUTTON_CENTER = 2;

class GameInput {
    updates: number = 0;
    mouse_position: number[] = [0.0, 0.0];
    scroll_delta_y: number = 0;

    // true: button was pressed, false: button was released, null: button state wasn't changed
    left_mouse_button: boolean|null = null;    
    right_mouse_button: boolean|null = null;
    center_mouse_button: boolean|null = null;

    keys: Map<string, boolean> = new Map();
    chars_buffer: string = "";
}

class Engine {
    ws: EngineWebSocket = new EngineWebSocket();

    game: GameInterface = new GameInterface();
    assets: EngineAssets = new EngineAssets();
    renderer: Renderer = new Renderer();
    input: GameInput = new GameInput();

    time: DOMHighResTimeStamp = 0;
    refresh_client: boolean = false;
    reload_client: boolean = false;
    reload: boolean = false;
    exit: boolean = false;
}

//
// Init
//

function init_handlers(engine: Engine) {
    const canvas = engine.renderer.canvas().element;
    const input_state = engine.input;

    function on_mouse_move(event: MouseEvent) {
        input_state.mouse_position[0] = event.clientX - canvas.offsetLeft;
        input_state.mouse_position[1] = event.clientY - canvas.offsetTop;
        input_state.updates |= UPDATE_MOUSE_POSITION;
    }

    function on_mouse_down(event: MouseEvent) {
        input_state.updates |= UPDATE_MOUSE_BUTTONS;

        if (event.button === 0) { input_state.left_mouse_button = true; }
        else if (event.button === 1) { input_state.center_mouse_button = true; }
        else if (event.button === 2) { input_state.right_mouse_button = true; }

        on_mouse_move(event);
        
        event.preventDefault();
    }

    function on_mouse_up(event: MouseEvent) {
        input_state.updates |= UPDATE_MOUSE_BUTTONS;

        if (event.button === 0) { input_state.left_mouse_button = false; }
        else if (event.button === 1) { input_state.center_mouse_button = false; }
        else if (event.button === 2) { input_state.right_mouse_button = false; }

        on_mouse_move(event);

        event.preventDefault();
    }

    function on_wheel(event: WheelEvent) {
        input_state.scroll_delta_y += event.deltaY;
        input_state.updates |= UPDATE_WHEEL;
    }

    canvas.addEventListener("mousemove", on_mouse_move)
    canvas.addEventListener("mousedown", on_mouse_down)
    canvas.addEventListener("mouseup", on_mouse_up)
    canvas.addEventListener("wheel", on_wheel)

    canvas.addEventListener("contextmenu", (event) => { event.preventDefault(); });

    window.addEventListener("keydown", (event) => {
        if (event.key.length == 1) {
            input_state.chars_buffer += event.key;
        }

        input_state.keys.set(event.code, true);
        input_state.updates |= UPDATE_KEYS;
    });

    window.addEventListener("keyup", (event) => {
        input_state.keys.set(event.code, false);
        input_state.updates |= UPDATE_KEYS;
    });

    document.getElementById("resetDemo")?.addEventListener("click", (event) => {
        engine.refresh_client = true;
    });

   
}

function init_touch_handlers(engine: Engine) {
    function touchHandler(event: TouchEvent) {
        var touches = event.changedTouches,
            first = touches[0],
            type = "";
        switch(event.type)
        {
            case "touchstart": type = "mousedown"; break;
            case "touchmove":  type = "mousemove"; break;        
            case "touchend":   type = "mouseup";   break;
            default:           return;
        }

        var simulatedEvent = document.createEvent("MouseEvent");
        simulatedEvent.initMouseEvent(type, true, true, window, 1, 
                                    first.screenX, first.screenY, 
                                    first.clientX, first.clientY, false, 
                                    false, false, false, 0/*left*/, null);

        first.target.dispatchEvent(simulatedEvent);
        event.preventDefault();
    }

    const canvas = engine.renderer.canvas().element;
    canvas.addEventListener("touchstart", touchHandler, true);
    canvas.addEventListener("touchmove", touchHandler, true);
    canvas.addEventListener("touchend", touchHandler, true);
    canvas.addEventListener("touchcancel", touchHandler, true);   
}

function start_client(engine: Engine): boolean {
    const canvas = engine.renderer.canvas();
    const params: GameInterfaceStartupParams = {
        screen_width: canvas.width,
        screen_height: canvas.height,
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
    init_touch_handlers(engine);

    engine.ws.open();

    (window as any).engine = engine;

    return engine;
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
        const canvas = engine.renderer.canvas();
        const width = canvas.width;
        const height = canvas.height;
        engine.game.resize(width, height)
    }
}

function game_input_updates(engine: Engine) {
    const inputs = engine.input;
    const game = engine.game.instance;

    if ((inputs.updates & UPDATE_MOUSE_POSITION) > 0) {
        game.update_mouse_position(inputs.mouse_position[0], inputs.mouse_position[1]);
    }

    if ((inputs.updates & UPDATE_WHEEL) > 0) {
        game.update_scroll_value(inputs.scroll_delta_y);
        inputs.scroll_delta_y = 0;
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

        game.push_chars_buffer(inputs.chars_buffer);
        inputs.chars_buffer = "";
    }

    inputs.keys.clear();
    inputs.updates = 0;
}

/// Execute the game logic of the client for the current frame
function game_updates(engine: Engine) {
    game_input_updates(engine)
    engine.game.instance.update(engine.time)
}

/// Reads the rendering updates generated by the game client
function renderer_updates(engine: Engine) {
    engine.renderer.update(engine.game);
}

function update(engine: Engine) {
    websocket_messages(engine);
    handle_resize(engine);
    game_updates(engine);
    renderer_updates(engine);
}

//
// Render
//

function render(engine: Engine) {
    engine.renderer.render(engine.time);
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

function refresh(engine: Engine) {
    engine.refresh_client = false;
    engine.game.free();
    start_client(engine);
    engine.renderer.refresh();
    console.log("Game client refreshed");
}

//
// Runtime
//


let boundedRun = () => {};

function run(engine: Engine) {
    if (engine.exit) {
        return;
    }

    engine.time = performance.now();

    update(engine);
    render(engine);

    if (engine.refresh_client) {
        refresh(engine);
    }

    if (engine.reload) {
        reload(engine)
            .then(() => requestAnimationFrame(boundedRun) );
    } else {
        requestAnimationFrame(boundedRun);
    }
}

//
// Startup
//

let engine: Engine|null = null
let toggle_demo_state = sessionStorage.getItem("retained_gui_toggle_demo_state") || "both";
let waiting_for_visible_demo = false;
const min_width_for_both = 600;

function updateBodyClasses() {
    const classes = document.body.classList;
    classes.remove("focus-article");
    classes.remove("focus-demo");
    if (toggle_demo_state === "article") {
        classes.add("focus-article");
    } else if (toggle_demo_state === "demo") {
        classes.add("focus-demo");
    }

    sessionStorage.setItem('retained_gui_toggle_demo_state', toggle_demo_state);
}

async function toggleDemo() {
    if (toggle_demo_state === "both") {
        toggle_demo_state = "article";
    } else if (toggle_demo_state === "article") {
        toggle_demo_state = "demo";
    } else if (toggle_demo_state === "demo") {
        if (document.body.offsetWidth < min_width_for_both) {
            toggle_demo_state = "article";
        } else {
            toggle_demo_state = "both";
        }
    }

    updateBodyClasses();

    await init_app();
}

function init_demo_toggle_handlers() {
    const body = document.body;
    if (body.offsetWidth < min_width_for_both) {
        toggle_demo_state = "article";
    }

    document.getElementById("toggleDemo")?.addEventListener("click", () => {
        toggleDemo();
    });

    window.addEventListener("resize", () => {
        if (body.offsetWidth < min_width_for_both && toggle_demo_state === "both") {
            toggle_demo_state = "article";
            updateBodyClasses();

        }
    });

    updateBodyClasses();
}

async function init_app() {
    if (engine) {
        return;
    }

    const demo = document.getElementById("demo") as HTMLCanvasElement;
    if (demo.clientWidth == 0 || demo.clientHeight == 0 && !waiting_for_visible_demo) {
        waiting_for_visible_demo = true;
        window.addEventListener("resize", init_app);
        return;
    }

    engine = await init();
    if (!engine) {
        console.log("Failed to initialize application");
        return;
    }

    boundedRun = run.bind(null, engine);
    boundedRun();

    if (waiting_for_visible_demo) {
        window.removeEventListener("resize", init_app);
        waiting_for_visible_demo = false;
    }
}

init_demo_toggle_handlers();
init_codeview();
init_app();
