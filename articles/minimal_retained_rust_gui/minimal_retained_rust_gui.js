let LAST_ERROR = null;
class Error {
    constructor(msg, tb) {
        this.message = msg;
        this.traceback = tb;
    }
}
function set_last_error(msg, tb) {
    LAST_ERROR = new Error(msg, null);
    console.log(LAST_ERROR);
}

const GAME_SRC_PATH = "/articles/minimal_retained_rust_gui/minimal_retained_rust_gui_demo.js";
class GameInterface {
    constructor() {
        this.reload_count = 0;
    }
    free() {
        if (this.instance) {
            this.instance.free();
        }
    }
    // @ts-ignore
    async init() {
        this.module = await import(GAME_SRC_PATH)
            .catch((e) => { set_last_error(`Failed to load the game client`); return null; });
        if (!this.module) {
            return false;
        }
        await this.module.default();
        return true;
    }
    start(assets, params) {
        const mod = this.module;
        const initial_data = mod.GameClientInit.new();
        // Config
        initial_data.max_texture_size(params.max_texture_size);
        initial_data.view_size(params.screen_width, params.screen_height);
        this.instance = mod.GameClient.initialize(initial_data);
        if (!this.instance) {
            set_last_error("Failed to start game client");
            return false;
        }
        return true;
    }
    async reload() {
        return true;
    }
}

async function fetch_text(url) {
    let response = await fetch(url)
        .catch((_) => { set_last_error(`Failed to fetch ${url}`); return null; });
    if (!response) {
        return null;
    }
    if (!response.ok) {
        set_last_error(`Failed to fetch ${url}`);
        return null;
    }
    return response.text();
}
async function fetch_blob(url) {
    let response = await fetch(url)
        .catch((_) => { set_last_error(`Failed to fetch ${url}`); return null; });
    if (!response) {
        return null;
    }
    if (!response.ok) {
        set_last_error(`Failed to fetch ${url}`);
        return null;
    }
    return response.blob();
}
async function fetch_arraybuffer(url) {
    let response = await fetch(url)
        .catch((_) => { set_last_error(`Failed to fetch ${url}`); return null; });
    if (!response) {
        return null;
    }
    if (!response.ok) {
        set_last_error(`Failed to fetch ${url}`);
        return null;
    }
    return response.arrayBuffer();
}

const ASSETS_BUNDLE = `
`;
class Shader {
    constructor(vertex, fragment) {
        this.vertex = vertex;
        this.fragment = fragment;
    }
}
class Texture {
    constructor(texture_id, path, bitmap) {
        this.id = texture_id;
        this.bitmap = bitmap;
        this.path = path;
    }
}
class EngineAssets {
    constructor() {
        this.bundle = ASSETS_BUNDLE;
        this.shaders = new Map();
        this.csv = new Map();
        this.fonts = new Map();
        this.textures = new Map();
        this.textures_by_id = [];
    }
    async init() {
        let bundle_loaded = await this.load_bundle();
        if (!bundle_loaded) {
            return false;
        }
        return true;
    }
    async load_bundle() {
        let split_line = "\n";
        if (this.bundle.indexOf("\r\n") != -1) {
            split_line = "\r\n";
        }
        const lines = this.bundle.split(split_line);
        let asset_loading_promises = [];
        let texture_id = 0;
        for (let line of lines) {
            if (line.length == 0) {
                continue;
            }
            const args = line.split(";");
            switch (args[0]) {
                case 'TEXTURE': {
                    const name = args[1];
                    const path = args[2];
                    asset_loading_promises.push(this.load_texture(texture_id, name, path));
                    texture_id += 1;
                    break;
                }
                case "CSV": {
                    const name = args[1];
                    const path = args[2];
                    asset_loading_promises.push(this.load_csv(name, path));
                    break;
                }
                case "SHADER": {
                    const name = args[1];
                    const vertex_path = args[2];
                    const fragment_path = args[3];
                    asset_loading_promises.push(this.load_shader(name, vertex_path, fragment_path));
                    break;
                }
                case "FONT": {
                    const name = args[1];
                    const path = args[2];
                    asset_loading_promises.push(this.load_font(name, path));
                    break;
                }
                default: {
                    console.log(`Warning: Unknown asset type ${args[0]} in bundle`);
                }
            }
        }
        const results = await Promise.all(asset_loading_promises);
        return results.indexOf(false) == -1;
    }
    async load_texture(texture_id, name, path) {
        const texture_blob = await fetch_blob(path);
        if (!texture_blob) {
            return false;
        }
        const bitmap = await createImageBitmap(texture_blob)
            .catch((_) => { set_last_error(`Failed to decode image ${path}`); return null; });
        if (!bitmap) {
            set_last_error(`Failed to load bitmap ${name}`);
            return false;
        }
        const texture = new Texture(texture_id, path, bitmap);
        this.textures.set(name, texture);
        this.textures_by_id[texture_id] = texture;
        return true;
    }
    async load_csv(name, path) {
        const csv_text = await fetch_text(path);
        if (!csv_text) {
            set_last_error(`Failed to load csv source for ${name}`);
            return false;
        }
        this.csv.set(name, csv_text);
        return true;
    }
    async load_shader(name, vertex_path, fragment_path) {
        const [vertex_text, fragment_text] = await Promise.all([
            fetch_text(vertex_path),
            fetch_text(fragment_path),
        ]);
        if (!vertex_text || !fragment_text) {
            set_last_error(`Failed to load shader source for ${name}`);
            return false;
        }
        this.shaders.set(name, new Shader(vertex_text, fragment_text));
        return true;
    }
    async load_font(name, path) {
        const data = await fetch_arraybuffer(path);
        if (!data) {
            return false;
        }
        this.fonts.set(name, data);
        return true;
    }
}

class RendererCanvas {
    constructor(container, element) {
        this.container = container;
        this.element = element;
        this.width = 0;
        this.height = 0;
    }
}
class Renderer {
    init() {
        if (!this.setup_canvas()) {
            return false;
        }
        if (!this.setup_context()) {
            return false;
        }
        if (!this.setup_framebuffer()) {
            return false;
        }
        return true;
    }
    init_default_resources(assets) {
        return true;
    }
    max_texture_size() {
        return this.ctx.getParameter(this.ctx.MAX_TEXTURE_SIZE);
    }
    //
    // Render
    //
    render() {
        const ctx = this.ctx;
        const canvas = this.canvas;
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 1.0]);
        ctx.bindFramebuffer(ctx.READ_FRAMEBUFFER, this.framebuffer);
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, null);
        ctx.blitFramebuffer(0, 0, canvas.width, canvas.height, 0, 0, canvas.width, canvas.height, ctx.COLOR_BUFFER_BIT, ctx.LINEAR);
    }
    //
    // Setup
    //
    setup_canvas() {
        const demo = document.getElementById("demo");
        const canvas_elem = document.getElementById("canvas");
        if (!canvas_elem) {
            set_last_error("Canvas element was not found");
            return false;
        }
        if (demo.clientWidth == 0 || demo.clientHeight == 0) {
            set_last_error("Canvas is not visible");
            return false;
        }
        this.canvas = new RendererCanvas(demo, canvas_elem);
        this.canvas.element.width = demo.clientWidth;
        this.canvas.element.height = demo.clientHeight;
        this.canvas.width = demo.clientWidth;
        this.canvas.height = demo.clientHeight;
        return true;
    }
    setup_context() {
        const canvas = this.canvas;
        const ctx = canvas.element.getContext("webgl2", {
            alpha: true,
            depth: false,
            stencil: false,
            antialias: false,
            premultipliedAlpha: false,
            preserveDrawingBuffer: false,
        });
        if (!ctx) {
            set_last_error("Webgl2 not supported");
            return false;
        }
        this.ctx = ctx;
        this.ctx.viewport(0, 0, canvas.width, canvas.height);
        return true;
    }
    setup_framebuffer() {
        const canvas = this.canvas;
        const ctx = this.ctx;
        const framebuffer = ctx.createFramebuffer();
        if (!framebuffer) {
            set_last_error("Failed to create the renderer framebuffer");
            return false;
        }
        const color = ctx.createRenderbuffer();
        if (!color) {
            set_last_error("Failed to create the renderer color render buffer");
            return false;
        }
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, framebuffer);
        ctx.bindRenderbuffer(ctx.RENDERBUFFER, color);
        ctx.renderbufferStorageMultisample(ctx.RENDERBUFFER, this.get_samples(), ctx.RGBA8, canvas.width, canvas.height);
        ctx.framebufferRenderbuffer(ctx.DRAW_FRAMEBUFFER, ctx.COLOR_ATTACHMENT0, ctx.RENDERBUFFER, color);
        this.framebuffer = framebuffer;
        this.color = color;
        return true;
    }
    get_samples() {
        let max_samples = this.ctx.getParameter(this.ctx.MAX_SAMPLES);
        function is_mobile() {
            let check = false;
            (function (a) { if (/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino|android|ipad|playbook|silk/i.test(a) || /1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(a.substr(0, 4)))
                check = true; })(navigator.userAgent || navigator.vendor || window.opera);
            return check;
        }
        // Don't use msaa on a mobile device
        if (is_mobile()) {
            max_samples = 1;
        }
        // We don't need more than 4x msaa
        if (max_samples > 4) {
            max_samples = 4;
        }
        return max_samples;
    }
}

const WEBSOCKET_HOST = "localhost:8001";
const VALID_MESSAGE_NAMES = ["FILE_CHANGED"];
class WebSocketMessage {
    constructor(name, data) {
        this.name = name;
        this.data = data;
    }
}
class EngineWebSocket {
    constructor() {
        this.socket = null;
        this.messages = [];
        this.messages_count = 0;
        this.opened = false;
    }
    async open() {
        let socket;
        try {
            socket = new WebSocket("ws://" + WEBSOCKET_HOST);
            socket.binaryType = "arraybuffer";
            socket.addEventListener("open", (event) => {
                this.opened = true;
            });
            socket.addEventListener('error', (event) => {
                console.log("Error while opening websocket connection!");
                console.log(event);
                this.opened = false;
                this.socket = null;
            });
            socket.addEventListener("message", (event) => {
                if (typeof event.data === "string") {
                    on_text_message(this, JSON.parse(event.data));
                }
                else {
                    on_bin_message(event.data);
                }
            });
            socket.addEventListener("close", (event) => {
                this.opened = false;
            });
        }
        catch {
            // No dev server
        }
    }
}
function on_text_message(ws, message) {
    if (message.name && message.data) {
        if (!VALID_MESSAGE_NAMES.includes(message.name)) {
            console.error("Unknown message:", message);
            return;
        }
        let ws_message = new WebSocketMessage(message.name, message.data);
        ws.messages[ws.messages_count] = ws_message;
        ws.messages_count += 1;
    }
    else {
        console.error("Unknown message:", message);
    }
}
function on_bin_message(data) {
}

class Engine {
    constructor() {
        this.ws = new EngineWebSocket();
        this.game = new GameInterface();
        this.assets = new EngineAssets();
        this.renderer = new Renderer();
        this.reload_client = false;
        this.reload = false;
        this.exit = false;
    }
}
function start_client(engine) {
    const params = {
        max_texture_size: engine.renderer.max_texture_size(),
        screen_width: engine.renderer.canvas.width,
        screen_height: engine.renderer.canvas.height,
    };
    return engine.game.start(engine.assets, params);
}
async function init() {
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
    engine.ws.open();
    window.engine = engine;
    return engine;
}
//
// Updates
//
function update(engine, time) {
}
//
// Render
//
function render(engine) {
    engine.renderer.render();
}
//
// Reload
//
async function reload(engine) {
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
let boundedRun = () => { };
function run(engine) {
    if (engine.exit) {
        return;
    }
    update(engine, performance.now());
    render(engine);
    if (engine.reload) {
        reload(engine)
            .then(() => requestAnimationFrame(boundedRun));
    }
    else {
        requestAnimationFrame(boundedRun);
    }
}
async function init_app() {
    const demo = document.getElementById("demo");
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
