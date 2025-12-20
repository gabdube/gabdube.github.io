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

function file_name(path) {
    return path.split(/[\\/]/).pop() || "";
}
function file_extension(path) {
    const lastDotIndex = path.lastIndexOf('.');
    if (lastDotIndex !== -1) {
        return path.slice(lastDotIndex + 1);
    }
    return '';
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
TEXTURE;atlas;./assets/atlas.png;
CSV;atlas_sprites;./assets/atlas.csv;
SHADER;gui;./assets/gui.vert.glsl;./assets/gui.frag.glsl;
MSDF_FONT;roboto;./assets/roboto.png;./assets/roboto.bin;
`;
class Shader {
    constructor(vertex, fragment) {
        this.vertex = vertex;
        this.fragment = fragment;
    }
}
class Texture {
    constructor(texture_id, bitmap) {
        this.id = texture_id;
        this.bitmap = bitmap;
    }
}
class MsdfFont {
    constructor(texture_id, atlas_data) {
        this.texture_id = texture_id;
        this.atlas_data = atlas_data;
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
                case "MSDF_FONT": {
                    const name = args[1];
                    const image_path = args[2];
                    const atlas_data_path = args[3];
                    asset_loading_promises.push(this.load_msdf_font(texture_id, name, image_path, atlas_data_path));
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
        const texture = new Texture(texture_id, bitmap);
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
    async load_msdf_font(texture_id, name, image_path, atlas_data_path) {
        const [texture_blob, atlas_data_buffer] = await Promise.all([
            fetch_blob(image_path),
            fetch_arraybuffer(atlas_data_path),
        ]);
        if (!texture_blob || !atlas_data_buffer) {
            return false;
        }
        const bitmap = await createImageBitmap(texture_blob)
            .catch((_) => { set_last_error(`Failed to decode image ${image_path}`); return null; });
        if (!bitmap) {
            set_last_error(`Failed to load msdf font ${name}`);
            return false;
        }
        const texture = new Texture(texture_id, bitmap);
        const font = new MsdfFont(texture_id, atlas_data_buffer);
        this.textures.set(name, texture);
        this.textures_by_id[texture_id] = texture;
        this.fonts.set(name, font);
        return true;
    }
}

const CODE_VIEW_DATA = {
    tabs: [],
    current_tab: -1, // -1 if demo
};
async function toggle_codeview() {
    const demo = document.getElementById("demo");
    const classes = demo.classList;
    if (!classes.contains("show-tabs")) {
        classes.add("show-tabs");
    }
    if (classes.contains("show-demo")) {
        classes.remove("show-demo");
        classes.add("show-code");
    }
}
async function toggle_demo() {
    const demo = document.getElementById("demo");
    const classes = demo.classList;
    if (classes.contains("show-code")) {
        classes.remove("show-code");
        classes.add("show-demo");
    }
}
function fetch_cache(target_url) {
    for (const tab of CODE_VIEW_DATA.tabs) {
        if (tab.url === target_url) {
            return tab;
        }
    }
    return null;
}
function refresh_tab_display() {
    const header = document.getElementById("demoheader");
    if (!header) {
        return;
    }
    while (header.children.length > 1) {
        header.removeChild(header.children[1]);
    }
    for (const [index, tab] of CODE_VIEW_DATA.tabs.entries()) {
        if (index == CODE_VIEW_DATA.current_tab) {
            tab.tab_element.classList.add("active");
        }
        else {
            tab.tab_element.classList.remove("active");
        }
        header.appendChild(tab.tab_element);
    }
    const show_demo = document.getElementById("demoheaderShowDemo");
    if (CODE_VIEW_DATA.current_tab === -1) {
        show_demo.classList.add("active");
    }
    else {
        show_demo.classList.remove("active");
    }
}
function show_tab(tab) {
    const codeview = document.getElementById("codeview");
    codeview.innerHTML = "";
    codeview.appendChild(tab.content);
    CODE_VIEW_DATA.current_tab = tab.index;
    codeview.scrollTo({ left: tab.scrollx, top: tab.scrolly });
    toggle_codeview();
}
function create_cross_svg() {
    const CROSS_SVG = `
<svg fill="rgb(27, 27, 27)" height="12px" width="12px" version="1.1" id="Capa_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" 
	 viewBox="0 0 460.775 460.775" xml:space="preserve">
<path d="M285.08,230.397L456.218,59.27c6.076-6.077,6.076-15.911,0-21.986L423.511,4.565c-2.913-2.911-6.866-4.55-10.992-4.55
	c-4.127,0-8.08,1.639-10.993,4.55l-171.138,171.14L59.25,4.565c-2.913-2.911-6.866-4.55-10.993-4.55
	c-4.126,0-8.08,1.639-10.992,4.55L4.558,37.284c-6.077,6.075-6.077,15.909,0,21.986l171.138,171.128L4.575,401.505
	c-6.074,6.077-6.074,15.911,0,21.986l32.709,32.719c2.911,2.911,6.865,4.55,10.992,4.55c4.127,0,8.08-1.639,10.994-4.55
	l171.117-171.12l171.118,171.12c2.913,2.911,6.866,4.55,10.993,4.55c4.128,0,8.081-1.639,10.992-4.55l32.709-32.719
	c6.074-6.075,6.074-15.909,0-21.986L285.08,230.397z"/>
</svg>
`;
    const parser = new DOMParser();
    const svgDoc = parser.parseFromString(CROSS_SVG, "image/svg+xml");
    const svgElement = svgDoc.documentElement;
    return svgElement;
}
async function add_tab(url) {
    async function generate_code_element(url) {
        let code_string = await (await fetch(url)).text();
        code_string = code_string.replaceAll("<", "&lt;");
        code_string = code_string.replaceAll(">", "&gt;");
        const code_elem = document.createElement("code");
        code_elem.innerHTML = code_string;
        hljs.highlightElement(code_elem);
        const content = document.createElement("pre");
        content.appendChild(code_elem);
        return content;
    }
    function generate_tab_element(name) {
        const range = document.createRange();
        const fragment = range.createContextualFragment(`<div class="demoheader-item"><span>${name}</span></div>`);
        return fragment.firstChild;
    }
    function select_tab(tab) {
        if (CODE_VIEW_DATA.current_tab !== tab.index) {
            CODE_VIEW_DATA.current_tab = tab.index;
            show_tab(tab);
            refresh_tab_display();
        }
    }
    async function close_tab(tab) {
        if (CODE_VIEW_DATA.current_tab == tab.index) {
            CODE_VIEW_DATA.current_tab -= 1;
            if (CODE_VIEW_DATA.current_tab >= 0) {
                show_tab(CODE_VIEW_DATA.tabs[CODE_VIEW_DATA.current_tab]);
            }
            else {
                toggle_demo();
            }
        }
        else if (CODE_VIEW_DATA.current_tab > tab.index) {
            CODE_VIEW_DATA.current_tab -= 1;
        }
        CODE_VIEW_DATA.tabs.splice(tab.index, 1);
        for (const [index, tab] of CODE_VIEW_DATA.tabs.entries()) {
            tab.index = index;
        }
        refresh_tab_display();
    }
    const index = CODE_VIEW_DATA.tabs.length;
    const name = file_name(url);
    const tab_element = generate_tab_element(name);
    const cross_svg = create_cross_svg();
    const content = await generate_code_element(url);
    const tab_data = {
        index,
        scrolly: 0,
        scrollx: 0,
        name,
        url,
        tab_element,
        content,
    };
    tab_element.title = url;
    tab_element.appendChild(cross_svg);
    tab_element.addEventListener("click", () => select_tab(tab_data));
    cross_svg.addEventListener("click", (event) => { close_tab(tab_data); event.stopPropagation(); });
    CODE_VIEW_DATA.tabs.push(tab_data);
    CODE_VIEW_DATA.current_tab = index;
    return tab_data;
}
function scroll_to_code(code) {
    const codeview = document.getElementById("codeview");
    const code_element = codeview.firstChild?.firstChild;
    if (!code_element) {
        console.error("Failed to find code element in codeview");
        return;
    }
    if (code.includes('\n') || code.includes('\r')) {
        console.error("scroll to node doesn't support newlines");
        return;
    }
    const children = code_element.childNodes;
    let found = null;
    let line_text = "";
    let i = 0;
    while (!found) {
        const child_item = children.item(i);
        if (!child_item) {
            break;
        }
        let text = child_item.textContent || "";
        let split_newlines = text.split("\n");
        if (split_newlines.length === 1) {
            i += 1;
            line_text += split_newlines[0];
            continue;
        }
        for (let j = 0; j < split_newlines.length - 1; j++) {
            line_text += split_newlines[j];
            if (line_text.length > 0) {
                const code_escaped = code.replaceAll("(", "\\(").replaceAll(")", "\\)");
                if ((line_text.match(code_escaped)?.length || 0) > 0) {
                    found = child_item;
                    break;
                }
                line_text = "";
            }
        }
        line_text = split_newlines[split_newlines.length - 1];
        i += 1;
    }
    // Hacky but good enough
    if (found) {
        let previous = found.previousSibling;
        if (!previous) {
            return;
        }
        while (!previous.scrollIntoView) {
            previous = previous.previousSibling;
        }
        previous.scrollIntoView({ behavior: "smooth" });
    }
}
async function load_code(link) {
    const url = link.dataset.url;
    if (!url) {
        console.error("No url defined in code link", link);
        return;
    }
    let tab = fetch_cache(url);
    if (!tab) {
        tab = await add_tab(url);
    }
    show_tab(tab);
    if (link.dataset.goto) {
        scroll_to_code(link.dataset.goto);
    }
    refresh_tab_display();
}
/// hljs is very slow: this function is 12 time more expensive to call than initializing the entire game demo, so we wrap it inside a timeout
/// to not cause a delay at page load time. I should replace hljs with a rust/wasm implementation.
function init_codeview() {
    const TIMEOUT = 500;
    setTimeout(() => {
        const content = document.getElementById("content");
        const demo = document.getElementById("demo");
        const codeview = document.getElementById("codeview");
        if (!content || !demo || !codeview) {
            return;
        }
        for (const link of content.getElementsByClassName("code-link")) {
            link.innerHTML = `${link.innerHTML}<span class="code-link-icon"></span>`;
            link.addEventListener("click", () => {
                if (demo.offsetWidth === 0) {
                    window.open(link.dataset.url2, "blank");
                }
                else {
                    load_code(link);
                    toggle_codeview();
                }
            });
        }
        for (const code of document.querySelectorAll('pre code')) {
            hljs.highlightElement(code);
        }
        const showdemo = document.getElementById("demoheaderShowDemo");
        if (showdemo) {
            showdemo.addEventListener("click", async () => {
                CODE_VIEW_DATA.current_tab = -1;
                toggle_demo();
                refresh_tab_display();
            });
        }
        codeview.addEventListener("scrollend", () => {
            const tab = CODE_VIEW_DATA.tabs[CODE_VIEW_DATA.current_tab];
            tab.scrolly = codeview.scrollTop;
            tab.scrollx = codeview.scrollLeft;
        });
    }, TIMEOUT);
}

/// This file was auto-generated
function getUint16Array(data, offset, count) {
    const values = [];
    for (let x = 0; x < count; x++) {
        values.push(data.getUint16(offset + (2 * x), true));
    }
    return values;
}
const ClearGui = 1;
const UpdateGui = 2;
const DrawGui = 3;
class UpdateGuiMessageParams {
    constructor(data) {
        this.data = data;
    }
    index_bytes_offset() { return this.data.getUint32(0, true); }
    index_bytes_size() { return this.data.getUint32(4, true); }
    vertex_bytes_offset() { return this.data.getUint32(8, true); }
    vertex_bytes_size() { return this.data.getUint32(12, true); }
}
class DrawGuiMessageParams {
    constructor(data) {
        this.data = data;
    }
    draw_count() { return this.data.getUint32(0, true); }
    index_bytes_offset() { return this.data.getUint32(4, true); }
    vertex_bytes_offset() { return this.data.getUint32(8, true); }
    image_texture() { return this.data.getUint32(12, true); }
    font_texture() { return this.data.getUint32(16, true); }
    scissor() { return getUint16Array(this.data, 20, 4); }
}
class GameUpdateIndex {
    constructor(data) {
        this.data = data;
    }
    messages_count() { return this.data.getUint32(0, true); }
    messages_size() { return this.data.getUint32(4, true); }
    messages_ptr() { return this.data.getUint32(8, true); }
    data_ptr() { return this.data.getUint32(12, true); }
}
class GameUpdateMessage {
    constructor(ty, params) {
        this.ty = ty;
        this.params = params;
    }
    update_gui() {
        return new UpdateGuiMessageParams(this.params);
    }
    draw_gui() {
        return new DrawGuiMessageParams(this.params);
    }
}
class GameUpdatesApi {
    constructor(buffer, output_index_ptr) {
        const index = new GameUpdateIndex(new DataView(buffer, output_index_ptr, 16));
        this.buffer = buffer;
        this.messages_count = index.messages_count();
        this.messages_size = index.messages_size();
        this.messages_ptr = index.messages_ptr();
        this.data_ptr = index.data_ptr();
    }
    get_message(index) {
        if (index >= this.messages_count) {
            console.error(`Tried to read message beyond total message count ${index} >= ${this.messages_count}`);
            return null;
        }
        const message_ptr = this.messages_ptr + (index * 32);
        const ty = new DataView(this.buffer, message_ptr, 4).getUint32(0, true);
        if (ty < ClearGui || ty > DrawGui) {
            console.error(`Received unknown message type: ${ty}`);
            return null;
        }
        const params = new DataView(this.buffer, message_ptr + 4, 28);
        return new GameUpdateMessage(ty, params);
    }
    get_data(offset, size) {
        return new Uint8Array(this.buffer, this.data_ptr + offset, size);
    }
}

const GAME_SRC_PATH = "./retained_gui.js";
class GameUpdates {
    constructor(buffer, output_index_ptr) {
        this.api = new GameUpdatesApi(buffer, output_index_ptr);
    }
    message_count() {
        return this.api.messages_count;
    }
    get_message(index) {
        return this.api.get_message(index);
    }
    get_data(offset, size) {
        return this.api.get_data(offset, size);
    }
}
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
        const initOutput = await this.module.default();
        this.memory = initOutput.memory;
        return true;
    }
    start(assets, params) {
        const mod = this.module;
        const initial_data = mod.GameClientInit.new();
        // Config
        initial_data.view_size(params.screen_width, params.screen_height);
        // Assets
        initial_data.set_assets_bundle(assets.bundle);
        for (const [csv_name, csv_value] of assets.csv.entries()) {
            initial_data.upload_text_asset(csv_name, csv_value);
        }
        for (const [font_name, font_value] of assets.fonts.entries()) {
            initial_data.upload_bin_asset(font_name, new Uint8Array(font_value.atlas_data));
        }
        const instance = mod.GameClient.initialize(initial_data);
        if (!instance) {
            set_last_error("Failed to start game client");
            return false;
        }
        this.instance = instance;
        return true;
    }
    async reload() {
        try {
            this.reload_count += 1;
            const saved = this.module.save(this.instance);
            this.module = await import(`${GAME_SRC_PATH}?v=${this.reload_count}`);
            const initOutput = await this.module.default();
            this.instance = this.module.load(saved);
            this.memory = initOutput.memory;
            return true;
        }
        catch (e) {
            console.log(e);
            return false;
        }
    }
    updates() {
        const output_index_ptr = this.instance.updates_ptr();
        return new GameUpdates(this.memory.buffer, output_index_ptr);
    }
    resize(width, height) {
        this.instance.resize(width, height);
    }
}

const NO_TEXTURE = 4_294_967_295;
const GUI_INDEX_BLOCK_SIZE = 2048;
const GUI_VERTEX_BLOCK_SIZE = 2048 * 10;
class RendererCanvas {
    constructor(container, element) {
        this.container = container;
        this.element = element;
        this.width = 0;
        this.height = 0;
    }
}
class RendererShaders {
    constructor() {
        this.gui_attributes = []; // position, texcoord, color
        this.gui_uniforms = []; // View position, View size
        this.gui = null;
    }
}
class GuiDrawCommand {
    constructor(vao, draw_count, index_bytes_offset, vertex_bytes_offset, font_texture, image_texture, scissor) {
        this.vao = vao;
        this.draw_count = draw_count;
        this.index_bytes_offset = index_bytes_offset;
        this.vertex_bytes_offset = vertex_bytes_offset;
        this.font_texture = font_texture;
        this.image_texture = image_texture;
        this.scissor = scissor;
    }
}
class RendererGui {
    constructor() {
        this.index = null;
        this.index_capacity = 0;
        this.index_size = 0;
        this.vertex = null;
        this.vertex_capacity = 0;
        this.vertex_size = 0;
        this.draw_commands_count = 0;
        this.draw_commands = [];
    }
}
class Renderer {
    constructor() {
        this._canvas = null;
        this._ctx = null;
        this._framebuffer = null;
        this.color = null;
        this.visible = false;
        this._assets = null;
        this.textures = [];
        this.default_texture = null;
        this.shaders = new RendererShaders();
        this.gui = new RendererGui();
        this.vao_pool = [];
        this.global_vao = null;
    }
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
        this.setup_base_context();
        this.setup_vao_pool();
        return true;
    }
    refresh() {
        this.handle_resize();
    }
    init_default_resources(assets) {
        this._assets = assets;
        if (!this.setup_shaders()) {
            return false;
        }
        if (!this.preload_textures()) {
            return false;
        }
        this.setup_gui();
        this.setup_uniforms();
        this.visible = true;
        return true;
    }
    canvas() {
        if (!this._canvas) {
            throw "Canvas not initialized";
        }
        return this._canvas;
    }
    ctx() {
        if (!this._ctx) {
            throw "Context not initialized";
        }
        return this._ctx;
    }
    framebuffer() {
        if (!this._framebuffer) {
            throw "Framebuffer not initialized";
        }
        return this._framebuffer;
    }
    assets() {
        if (!this._assets) {
            throw "Engine assets not set";
        }
        return this._assets;
    }
    //
    // Resize
    //
    handle_resize_framebuffer() {
        const canvas = this.canvas();
        const display_width = canvas.container.clientWidth;
        const display_height = canvas.container.clientHeight;
        if (this.visible && display_width == canvas.width && display_height == canvas.height) {
            return false;
        }
        if (display_width === 0.0 || display_height === 0.0) {
            this.visible = false;
            canvas.width = 0;
            canvas.height = 0;
            return false;
        }
        if (!this.color) {
            throw "Color buffer not initialized";
        }
        const ctx = this.ctx();
        canvas.element.width = display_width;
        canvas.element.height = display_height;
        canvas.width = display_width;
        canvas.height = display_height;
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer());
        ctx.bindRenderbuffer(ctx.RENDERBUFFER, this.color);
        ctx.renderbufferStorageMultisample(ctx.RENDERBUFFER, this.get_samples(), ctx.RGBA8, canvas.width, canvas.height);
        ctx.framebufferRenderbuffer(ctx.DRAW_FRAMEBUFFER, ctx.COLOR_ATTACHMENT0, ctx.RENDERBUFFER, this.color);
        ctx.viewport(0, 0, canvas.width, canvas.height);
        this.visible = true;
        return true;
    }
    handle_resize_uniforms() {
        const ctx = this.ctx();
        const canvas = this.canvas();
        const shaders = this.shaders;
        const size = new Float32Array([canvas.width, canvas.height]);
        const size_uniforms = [
            [shaders.gui, shaders.gui_uniforms[0]],
        ];
        for (let [shader, uniform] of size_uniforms) {
            ctx.useProgram(shader);
            ctx.uniform2fv(uniform, size);
        }
    }
    handle_resize() {
        if (!this.handle_resize_framebuffer()) {
            return false;
        }
        this.handle_resize_uniforms();
        return true;
    }
    //
    // Update
    //
    clear_gui() {
        const gui = this.gui;
        for (let i = 0; i < gui.draw_commands_count; i++) {
            this.vao_pool.push(gui.draw_commands[i].vao);
        }
        gui.draw_commands_count = 0;
    }
    update_gui(updates, params) {
        const ctx = this.ctx();
        const gui = this.gui;
        const index_offset = params.index_bytes_offset();
        const index_size = params.index_bytes_size();
        const vertex_offset = params.vertex_bytes_offset();
        const vertex_size = params.vertex_bytes_size();
        // Realloc
        if (index_size > gui.index_capacity && gui.index) {
            const new_capacity = index_size + GUI_INDEX_BLOCK_SIZE;
            gui.index = realloc_buffer(ctx, gui.index, ctx.ELEMENT_ARRAY_BUFFER, gui.index_capacity, new_capacity);
            gui.index_capacity = new_capacity;
        }
        if (vertex_size > gui.vertex_capacity && gui.vertex) {
            const new_capacity = vertex_size + GUI_VERTEX_BLOCK_SIZE;
            gui.vertex = realloc_buffer(ctx, gui.vertex, ctx.ARRAY_BUFFER, gui.vertex_capacity, new_capacity);
            gui.vertex_capacity = new_capacity;
        }
        // Data
        ctx.bindVertexArray(this.global_vao);
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.gui.index);
        ctx.bufferSubData(ctx.ELEMENT_ARRAY_BUFFER, 0, updates.get_data(index_offset, index_size));
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.gui.vertex);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, updates.get_data(vertex_offset, vertex_size));
        ctx.bindVertexArray(null);
        gui.vertex_size = vertex_size;
        gui.index_size = index_size;
        // Write vao
        for (let i = 0; i < gui.draw_commands_count; i++) {
            const cmd = gui.draw_commands[i];
            this.write_gui_vao(cmd.vao, cmd.vertex_bytes_offset);
        }
    }
    draw_gui(params) {
        const ctx = this.ctx();
        const gui = this.gui;
        const draw_count = params.draw_count();
        const vertex_bytes_offset = params.vertex_bytes_offset();
        const index_bytes_offset = params.index_bytes_offset();
        const font_texture_id = params.font_texture();
        const font_texture = this.get_texture(font_texture_id);
        const image_texture_id = params.image_texture();
        const image_texture = this.get_texture(image_texture_id);
        const scissor = params.scissor();
        scissor[1] = this.canvas().height - scissor[1] - scissor[3]; // Convert the scissor coordinates to webgl
        const vao = next_vao(ctx, this.vao_pool);
        gui.draw_commands[gui.draw_commands_count] = new GuiDrawCommand(vao, draw_count, index_bytes_offset, vertex_bytes_offset, font_texture, image_texture, scissor);
        gui.draw_commands_count += 1;
    }
    prepare_updates() {
        this.ctx().bindVertexArray(null);
    }
    update(game) {
        this.prepare_updates();
        const updates = game.updates();
        const message_count = updates.message_count();
        for (let i = 0; i < message_count; i++) {
            const message = updates.get_message(i);
            if (!message) {
                break;
            }
            switch (message.ty) {
                case ClearGui: {
                    this.clear_gui();
                    break;
                }
                case UpdateGui: {
                    this.update_gui(updates, message.update_gui());
                    break;
                }
                case DrawGui: {
                    this.draw_gui(message.draw_gui());
                    break;
                }
            }
        }
    }
    //
    // Render
    //
    render_gui() {
        const ctx = this.ctx();
        const gui = this.gui;
        if (gui.draw_commands_count == 0 || !this.shaders.gui) {
            return;
        }
        ctx.enable(ctx.SCISSOR_TEST);
        ctx.useProgram(this.shaders.gui);
        for (let i = 0; i < gui.draw_commands_count; i++) {
            const cmd = gui.draw_commands[i];
            ctx.bindVertexArray(cmd.vao);
            const [x, y, width, height] = cmd.scissor;
            ctx.scissor(x, y, width, height);
            ctx.activeTexture(ctx.TEXTURE0);
            ctx.bindTexture(ctx.TEXTURE_2D, cmd.image_texture);
            ctx.activeTexture(ctx.TEXTURE1);
            ctx.bindTexture(ctx.TEXTURE_2D, cmd.font_texture);
            ctx.drawElements(ctx.TRIANGLES, cmd.draw_count, ctx.UNSIGNED_SHORT, cmd.index_bytes_offset);
        }
        ctx.disable(ctx.SCISSOR_TEST);
    }
    render(time) {
        if (!this.visible) {
            return;
        }
        const ctx = this.ctx();
        const canvas = this.canvas();
        const framebuffer = this.framebuffer();
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 0.0]);
        this.render_gui();
        ctx.bindFramebuffer(ctx.READ_FRAMEBUFFER, framebuffer);
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, null);
        ctx.blitFramebuffer(0, 0, canvas.width, canvas.height, 0, 0, canvas.width, canvas.height, ctx.COLOR_BUFFER_BIT, ctx.LINEAR);
    }
    //
    // Setup
    //
    setup_base_context() {
        const ctx = this.ctx();
        ctx.disable(ctx.CULL_FACE);
        ctx.enable(ctx.BLEND);
        ctx.blendFunc(ctx.ONE, ctx.ONE_MINUS_SRC_ALPHA);
        ctx.blendEquation(ctx.FUNC_ADD);
    }
    setup_canvas() {
        const demo = document.getElementById("demo");
        const canvas_elem = document.getElementById("canvas");
        if (!canvas_elem) {
            set_last_error("Canvas element was not found");
            return false;
        }
        if (canvas_elem.clientWidth == 0 || canvas_elem.clientHeight == 0) {
            set_last_error("Canvas is not visible");
            return false;
        }
        this._canvas = new RendererCanvas(demo, canvas_elem);
        this._canvas.element.width = canvas_elem.clientWidth;
        this._canvas.element.height = canvas_elem.clientHeight;
        this._canvas.width = canvas_elem.clientWidth;
        this._canvas.height = canvas_elem.clientHeight;
        return true;
    }
    setup_context() {
        const canvas = this.canvas();
        const ctx = canvas.element.getContext("webgl2", {
            alpha: true,
            depth: false,
            stencil: false,
            antialias: false,
            premultipliedAlpha: true,
            preserveDrawingBuffer: false,
        });
        if (!ctx) {
            set_last_error("Webgl2 not supported");
            return false;
        }
        this._ctx = ctx;
        this._ctx.viewport(0, 0, canvas.width, canvas.height);
        return true;
    }
    setup_framebuffer() {
        const canvas = this.canvas();
        const ctx = this.ctx();
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
        this._framebuffer = framebuffer;
        this.color = color;
        return true;
    }
    setup_shaders() {
        const ctx = this.ctx();
        const assets = this.assets();
        const shaders = this.shaders;
        const gui = build_shader(ctx, assets, "gui", ["in_position", "in_texcoord", "in_color", "in_data"], ["view_size", "image_texture", "font_texture"]);
        if (gui) {
            shaders.gui = gui.program;
            shaders.gui_attributes = gui.attributes;
            shaders.gui_uniforms = gui.uniforms;
        }
        else {
            return false;
        }
        return true;
    }
    setup_vao_pool() {
        const ctx = this.ctx();
        for (let i = 0; i < 16; i++) {
            this.vao_pool.push(ctx.createVertexArray());
        }
        this.global_vao = ctx.createVertexArray();
    }
    preload_textures() {
        const to_preload = ["atlas", "roboto"];
        for (let name of to_preload) {
            const texture = this.assets().textures.get(name);
            if (!texture) {
                set_last_error(`Failed to preload texture ${name}: missing texture in assets`);
                return false;
            }
            this.create_texture(texture.id);
        }
        // Default texture
        this.default_texture = create_default_texture(this.ctx());
        return true;
    }
    setup_gui() {
        const ctx = this.ctx();
        const gui = this.gui;
        gui.index = ctx.createBuffer();
        gui.index_capacity = GUI_INDEX_BLOCK_SIZE;
        gui.vertex = ctx.createBuffer();
        gui.vertex_capacity = GUI_VERTEX_BLOCK_SIZE;
        ctx.bindVertexArray(this.global_vao);
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, gui.index);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, gui.index_capacity, ctx.STATIC_DRAW);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, gui.vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, gui.vertex_capacity, ctx.STATIC_DRAW);
    }
    setup_uniforms() {
        const ctx = this.ctx();
        const canvas = this.canvas();
        const shaders = this.shaders;
        const size = new Float32Array([canvas.width, canvas.height]);
        ctx.useProgram(shaders.gui);
        ctx.uniform2fv(shaders.gui_uniforms[0], size); // view_size
        ctx.uniform1i(shaders.gui_uniforms[1], 0); // image_texture
        ctx.uniform1i(shaders.gui_uniforms[2], 1); // font_texture
    }
    get_samples() {
        let max_samples = this.ctx().getParameter(this.ctx().MAX_SAMPLES);
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
    write_gui_vao(vao, vertex_bytes_offset) {
        const GUI_VERTEX_SIZE = 24;
        const ctx = this.ctx();
        const gui = this.gui;
        const [position, texcoord, color, data] = this.shaders.gui_attributes;
        ctx.bindVertexArray(vao);
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, gui.index);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, gui.vertex);
        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, GUI_VERTEX_SIZE, vertex_bytes_offset + 0);
        ctx.enableVertexAttribArray(texcoord);
        ctx.vertexAttribPointer(texcoord, 2, ctx.FLOAT, false, GUI_VERTEX_SIZE, vertex_bytes_offset + 8);
        ctx.enableVertexAttribArray(color);
        ctx.vertexAttribPointer(color, 4, ctx.UNSIGNED_BYTE, true, GUI_VERTEX_SIZE, vertex_bytes_offset + 16);
        ctx.enableVertexAttribArray(data);
        ctx.vertexAttribIPointer(data, 1, ctx.UNSIGNED_INT, GUI_VERTEX_SIZE, vertex_bytes_offset + 20);
        ctx.bindVertexArray(null);
    }
    //
    // Helpers
    //
    get_texture(texture_id) {
        if (texture_id < this.textures.length) {
            const texture = this.textures[texture_id];
            if (texture) {
                return texture;
            }
        }
        else if (texture_id === NO_TEXTURE) {
            if (!this.default_texture) {
                throw "Default texture not initialized";
            }
            return this.default_texture;
        }
        return this.create_texture(texture_id);
    }
    create_texture(texture_id) {
        const texture_asset = this.assets().textures_by_id[texture_id];
        if (!texture_asset) {
            if (!this.default_texture) {
                throw "Default texture not initialized";
            }
            console.error(`Unknown texture ID ${texture_id}, returning default texture`);
            return this.default_texture;
        }
        while (this.textures.length < texture_id) {
            this.textures.push(null);
        }
        const texture = create_texture_rgba(this.ctx(), texture_asset);
        this.textures[texture_id] = texture;
        return texture;
    }
}
function build_shader(ctx, assets, shader_name, attributes_names, uniforms_names) {
    const shader_source = assets.shaders.get(shader_name);
    if (!shader_source) {
        set_last_error(`Failed to find shader source for shader "${shader_name}" in assets`);
        return;
    }
    const vert = create_shader(ctx, ctx.VERTEX_SHADER, shader_source.vertex);
    const frag = create_shader(ctx, ctx.FRAGMENT_SHADER, shader_source.fragment);
    if (!vert || !frag) {
        set_last_error(`Failed to create shaders for "${shader_name}"`);
        return;
    }
    const program = create_program(ctx, vert, frag);
    if (!program) {
        set_last_error(`Failed to compile shaders for "${shader_name}"`);
        return;
    }
    const attributes = [];
    for (let attribute_name of attributes_names) {
        const loc = ctx.getAttribLocation(program, attribute_name);
        if (loc == -1) {
            set_last_error(`Unkown attribute "${attribute_name}" in shader "${shader_name}"`);
            return;
        }
        attributes.push(loc);
    }
    const uniforms = [];
    for (let uniform_name of uniforms_names) {
        const loc = ctx.getUniformLocation(program, uniform_name);
        if (!loc) {
            set_last_error(`Unkown uniform "${uniform_name}" in shader "${shader_name}"`);
            return;
        }
        uniforms.push(loc);
    }
    ctx.deleteShader(vert);
    ctx.deleteShader(frag);
    return {
        program,
        attributes,
        uniforms,
    };
}
function create_shader(ctx, type, source) {
    const shader = ctx.createShader(type);
    ctx.shaderSource(shader, source);
    ctx.compileShader(shader);
    const success = ctx.getShaderParameter(shader, ctx.COMPILE_STATUS);
    if (success) {
        return shader;
    }
    console.log(ctx.getShaderInfoLog(shader));
    ctx.deleteShader(shader);
}
function create_program(ctx, vertexShader, fragmentShader) {
    const program = ctx.createProgram();
    ctx.attachShader(program, vertexShader);
    ctx.attachShader(program, fragmentShader);
    ctx.linkProgram(program);
    const success = ctx.getProgramParameter(program, ctx.LINK_STATUS);
    if (success) {
        return program;
    }
    console.log(ctx.getProgramInfoLog(program));
    ctx.deleteProgram(program);
}
function create_default_texture(ctx) {
    const dimension = 4;
    const pixel_size = 4;
    const byte_size = dimension * dimension * pixel_size;
    const data = new Uint8Array(byte_size);
    for (let i = 0; i < byte_size; i += 4) {
        data[i + 0] = 255;
        data[i + 1] = 0;
        data[i + 2] = 255;
        data[i + 3] = 255;
    }
    const texture = ctx.createTexture();
    ctx.bindTexture(ctx.TEXTURE_2D, texture);
    ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_MAG_FILTER, ctx.NEAREST);
    ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.NEAREST);
    ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.REPEAT);
    ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.REPEAT);
    ctx.texStorage2D(ctx.TEXTURE_2D, 1, ctx.RGBA8, dimension, dimension);
    ctx.texSubImage2D(ctx.TEXTURE_2D, 0, 0, 0, dimension, dimension, ctx.RGBA, ctx.UNSIGNED_BYTE, data);
    return texture;
}
function create_texture_rgba(ctx, cpu_texture) {
    const bitmap = cpu_texture.bitmap;
    const texture = ctx.createTexture();
    ctx.bindTexture(ctx.TEXTURE_2D, texture);
    ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_MAG_FILTER, ctx.LINEAR);
    ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.LINEAR);
    ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE);
    ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE);
    ctx.texStorage2D(ctx.TEXTURE_2D, 1, ctx.RGBA8, bitmap.width, bitmap.height);
    ctx.texSubImage2D(ctx.TEXTURE_2D, 0, 0, 0, bitmap.width, bitmap.height, ctx.RGBA, ctx.UNSIGNED_BYTE, bitmap);
    return texture;
}
function realloc_buffer(ctx, buffer, target, old_capacity, new_capacity, copy_data) {
    const new_buffer = ctx.createBuffer();
    ctx.bindBuffer(target, new_buffer);
    ctx.bufferData(target, new_capacity, ctx.DYNAMIC_DRAW);
    {
        ctx.bindBuffer(ctx.COPY_READ_BUFFER, buffer);
        ctx.bindBuffer(ctx.COPY_WRITE_BUFFER, new_buffer);
        ctx.copyBufferSubData(ctx.COPY_READ_BUFFER, ctx.COPY_WRITE_BUFFER, 0, 0, old_capacity);
        ctx.bindBuffer(ctx.COPY_READ_BUFFER, null);
        ctx.bindBuffer(ctx.COPY_WRITE_BUFFER, null);
    }
    ctx.deleteBuffer(buffer);
    return new_buffer;
}
function next_vao(ctx, pool) {
    if (pool.length === 0) {
        for (let i = 0; i < 16; i++) {
            pool.push(ctx.createVertexArray());
        }
    }
    return pool.pop();
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

const UPDATE_MOUSE_POSITION = 0b0001;
const UPDATE_MOUSE_BUTTONS = 0b0010;
const UPDATE_KEYS = 0b0100;
const UPDATE_WHEEL = 0b1000;
// Matches `MouseButton` in `game\src\inputs.rs`
const MOUSE_BUTTON_LEFT = 0;
const MOUSE_BUTTON_RIGHT = 1;
const MOUSE_BUTTON_CENTER = 2;
class GameInput {
    constructor() {
        this.updates = 0;
        this.mouse_position = [0.0, 0.0];
        this.scroll_delta_y = 0;
        // true: button was pressed, false: button was released, null: button state wasn't changed
        this.left_mouse_button = null;
        this.right_mouse_button = null;
        this.center_mouse_button = null;
        this.keys = new Map();
        this.chars_buffer = "";
    }
}
class Engine {
    constructor() {
        this.ws = new EngineWebSocket();
        this.game = new GameInterface();
        this.assets = new EngineAssets();
        this.renderer = new Renderer();
        this.input = new GameInput();
        this.time = 0;
        this.refresh_client = false;
        this.reload_client = false;
        this.reload = false;
        this.exit = false;
    }
}
//
// Init
//
function init_handlers(engine) {
    const canvas = engine.renderer.canvas().element;
    const input_state = engine.input;
    function on_mouse_move(event) {
        input_state.mouse_position[0] = event.clientX - canvas.offsetLeft;
        input_state.mouse_position[1] = event.clientY - canvas.offsetTop;
        input_state.updates |= UPDATE_MOUSE_POSITION;
    }
    function on_mouse_down(event) {
        input_state.updates |= UPDATE_MOUSE_BUTTONS;
        if (event.button === 0) {
            input_state.left_mouse_button = true;
        }
        else if (event.button === 1) {
            input_state.center_mouse_button = true;
        }
        else if (event.button === 2) {
            input_state.right_mouse_button = true;
        }
        on_mouse_move(event);
        event.preventDefault();
    }
    function on_mouse_up(event) {
        input_state.updates |= UPDATE_MOUSE_BUTTONS;
        if (event.button === 0) {
            input_state.left_mouse_button = false;
        }
        else if (event.button === 1) {
            input_state.center_mouse_button = false;
        }
        else if (event.button === 2) {
            input_state.right_mouse_button = false;
        }
        on_mouse_move(event);
        event.preventDefault();
    }
    function on_wheel(event) {
        input_state.scroll_delta_y += event.deltaY;
        input_state.updates |= UPDATE_WHEEL;
    }
    canvas.addEventListener("mousemove", on_mouse_move);
    canvas.addEventListener("mousedown", on_mouse_down);
    canvas.addEventListener("mouseup", on_mouse_up);
    canvas.addEventListener("wheel", on_wheel);
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
function init_touch_handlers(engine) {
    function touchHandler(event) {
        var touches = event.changedTouches, first = touches[0], type = "";
        switch (event.type) {
            case "touchstart":
                type = "mousedown";
                break;
            case "touchmove":
                type = "mousemove";
                break;
            case "touchend":
                type = "mouseup";
                break;
            default: return;
        }
        var simulatedEvent = document.createEvent("MouseEvent");
        simulatedEvent.initMouseEvent(type, true, true, window, 1, first.screenX, first.screenY, first.clientX, first.clientY, false, false, false, false, 0 /*left*/, null);
        first.target.dispatchEvent(simulatedEvent);
        event.preventDefault();
    }
    const canvas = engine.renderer.canvas().element;
    canvas.addEventListener("touchstart", touchHandler, true);
    canvas.addEventListener("touchmove", touchHandler, true);
    canvas.addEventListener("touchend", touchHandler, true);
    canvas.addEventListener("touchcancel", touchHandler, true);
}
function start_client(engine) {
    const canvas = engine.renderer.canvas();
    const params = {
        screen_width: canvas.width,
        screen_height: canvas.height,
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
    init_handlers(engine);
    init_touch_handlers(engine);
    engine.ws.open();
    window.engine = engine;
    return engine;
}
//
// Updates
//
function on_file_changed(engine, message) {
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
function websocket_messages(engine) {
    const ws = engine.ws;
    if (!ws.open) {
        // We're using a static client with no dev server
        return;
    }
    for (let i = 0; i < ws.messages_count; i++) {
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
function handle_resize(engine) {
    if (engine.renderer.handle_resize()) {
        const canvas = engine.renderer.canvas();
        const width = canvas.width;
        const height = canvas.height;
        engine.game.resize(width, height);
    }
}
function game_input_updates(engine) {
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
function game_updates(engine) {
    game_input_updates(engine);
    engine.game.instance.update(engine.time);
}
/// Reads the rendering updates generated by the game client
function renderer_updates(engine) {
    engine.renderer.update(engine.game);
}
function update(engine) {
    websocket_messages(engine);
    handle_resize(engine);
    game_updates(engine);
    renderer_updates(engine);
}
//
// Render
//
function render(engine) {
    engine.renderer.render(engine.time);
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
function refresh(engine) {
    engine.refresh_client = false;
    engine.game.free();
    start_client(engine);
    engine.renderer.refresh();
    console.log("Game client refreshed");
}
//
// Runtime
//
let boundedRun = () => { };
function run(engine) {
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
            .then(() => requestAnimationFrame(boundedRun));
    }
    else {
        requestAnimationFrame(boundedRun);
    }
}
//
// Startup
//
let engine = null;
let toggle_demo_state = sessionStorage.getItem("retained_gui_toggle_demo_state") || "both";
let waiting_for_visible_demo = false;
const min_width_for_both = 600;
function updateBodyClasses() {
    const classes = document.body.classList;
    classes.remove("focus-article");
    classes.remove("focus-demo");
    if (toggle_demo_state === "article") {
        classes.add("focus-article");
    }
    else if (toggle_demo_state === "demo") {
        classes.add("focus-demo");
    }
    sessionStorage.setItem('retained_gui_toggle_demo_state', toggle_demo_state);
}
async function toggleDemo() {
    if (toggle_demo_state === "both") {
        toggle_demo_state = "article";
    }
    else if (toggle_demo_state === "article") {
        toggle_demo_state = "demo";
    }
    else if (toggle_demo_state === "demo") {
        if (document.body.offsetWidth < min_width_for_both) {
            toggle_demo_state = "article";
        }
        else {
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
    const demo = document.getElementById("demo");
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
