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

/// Interface between the wasm client and the engine
const GAME_SRC_PATH = "/articles/navmesh_pathfinding/navmesh_pathfinding_demo.js";
class GameUpdates {
    constructor(protocol, buffer, output_index_ptr) {
        this.protocol = protocol;
        this.buffer = buffer;
        const index = new this.protocol.OutputIndex(buffer, output_index_ptr);
        this.messages_count = index.messages_count();
        this.messages_size = index.messages_size();
        this.messages_ptr = index.messages_ptr();
        this.base_data_ptr = index.data_ptr();
    }
    get_message(index) {
        const offset = this.messages_ptr + (index * this.messages_size);
        return new this.protocol.OutputMessage(this.buffer, offset);
    }
    get_data(offset, size) {
        return this.buffer.slice(this.base_data_ptr + offset, this.base_data_ptr + offset + size);
    }
}
class GameInterface {
    constructor() {
        this.protocol = null;
        this.reload_count = 0;
    }
    // @ts-ignore
    async init() {
        this.module = await import(GAME_SRC_PATH)
            .catch((e) => { set_last_error(`Failed to load the game client`); return null; });
        if (!this.module) {
            return false;
        }
        await this.module.default();
        // Protocol is a javascript module generated that the game client that informs the engine of the client data layout
        const proto = this.module.protocol();
        const blob = new Blob([proto], { type: 'application/javascript' });
        const moduleUrl = URL.createObjectURL(blob);
        this.protocol = await import(moduleUrl);
        return true;
    }
    start(assets) {
        const mod = this.module;
        const initial_data = mod.GameClientInit.new();
        initial_data.set_assets_bundle(assets.bundle);
        for (const [csv_name, csv_value] of assets.csv.entries()) {
            initial_data.upload_text_asset(csv_name, csv_value);
        }
        this.instance = mod.GameClient.initialize(initial_data);
        if (!this.instance) {
            set_last_error("Failed to start game client");
            return false;
        }
        return true;
    }
    async reload() {
        try {
            this.reload_count += 1;
            const saved = this.module.save(this.instance);
            this.module = await import(`${GAME_SRC_PATH}?v=${this.reload_count}`);
            await this.module.default();
            this.instance = this.module.load(saved);
            return true;
        }
        catch (e) {
            console.log(e);
            return false;
        }
    }
    updates() {
        const buffer = this.get_memory();
        const output_index_ptr = this.instance.updates_ptr();
        return new GameUpdates(this.protocol, buffer, output_index_ptr);
    }
    get_memory() {
        if (this.module) {
            // If the module was already initialized, this only returns the wasm memory
            return this.module.initSync().memory.buffer;
        }
        else {
            throw "Client module is not loaded";
        }
    }
}

const ASSETS_BUNDLE = `
TEXTURE;atlas;assets/atlas.png;
TEXTURE;terrain;assets/terrain.png;
CSV;atlas_sprites;assets/atlas.csv;
SHADER;sprites;assets/sprites.vert.glsl;assets/sprites.frag.glsl;
SHADER;terrain;assets/terrain.vert.glsl;assets/terrain.frag.glsl;
SHADER;debug;assets/debug.vert.glsl;assets/debug.frag.glsl;
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
}

const BASE_SPRITES_CAPACITY = 1024 * 2;
const BASE_TERRAIN_CAPACITY = 1024 * 10;
const BASE_DEBUG_CAPACITY = 1024;
class RendererCanvas {
    constructor(container, element) {
        this.container = container;
        this.element = element;
        this.width = 0;
        this.height = 0;
    }
}
class SpritesBuffer {
    constructor() {
        this.draw_count = 0;
        this.draw = [];
    }
}
class SpritesDraw {
    constructor() {
        this.instance_count = 0;
        this.vao = null;
        this.texture = null;
    }
}
class Terrain {
}
class Debug {
}
class RendererShaders {
}
class Renderer {
    constructor() {
        this.visible = false;
        this.shaders = new RendererShaders();
        this.textures = [];
        this.sprites = new SpritesBuffer();
        this.terrain = new Terrain();
        this.debug = new Debug();
        this.vao_pool_next = 0;
        this.vao_pool = [];
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
        return true;
    }
    init_default_resources(assets) {
        this.assets = assets;
        if (!this.setup_shaders()) {
            return false;
        }
        if (!this.preload_textures()) {
            return false;
        }
        this.setup_pools();
        this.setup_terrain();
        this.setup_sprites();
        this.setup_debug();
        this.setup_uniforms();
        this.visible = true;
        return true;
    }
    //
    // Resize
    //
    handle_resize_framebuffer() {
        const canvas = this.canvas;
        const display_width = canvas.container.clientWidth;
        const display_height = canvas.container.clientHeight;
        if (display_width == canvas.width && display_height == canvas.height) {
            return false;
        }
        if (display_width == 0.0 || display_height == 0.0) {
            this.visible = false;
            return false;
        }
        const ctx = this.ctx;
        canvas.element.width = display_width;
        canvas.element.height = display_height;
        canvas.width = display_width;
        canvas.height = display_height;
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.bindRenderbuffer(ctx.RENDERBUFFER, this.color);
        ctx.renderbufferStorageMultisample(ctx.RENDERBUFFER, this.get_samples(), ctx.RGBA8, canvas.width, canvas.height);
        ctx.framebufferRenderbuffer(ctx.DRAW_FRAMEBUFFER, ctx.COLOR_ATTACHMENT0, ctx.RENDERBUFFER, this.color);
        ctx.viewport(0, 0, canvas.width, canvas.height);
        this.visible = true;
        return true;
    }
    handle_resize_uniforms() {
        const ctx = this.ctx;
        const size = new Float32Array([this.canvas.width, this.canvas.height]);
        const size_uniforms = [
            [this.shaders.sprites, this.shaders.sprites_uniforms[1]],
            [this.shaders.terrain, this.shaders.terrain_uniforms[1]],
            [this.shaders.debug, this.shaders.debug_uniforms[1]],
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
    // Updates
    //
    next_vao() {
        const vao_index = this.vao_pool_next;
        if (vao_index >= this.vao_pool.length) {
            this.vao_pool.push(this.ctx.createVertexArray());
        }
        this.vao_pool_next += 1;
        return this.vao_pool[vao_index];
    }
    next_sprite_draw() {
        const sprites = this.sprites;
        const draw_index = sprites.draw_count;
        if (draw_index >= sprites.draw.length) {
            sprites.draw.push(new SpritesDraw());
        }
        sprites.draw_count += 1;
        return sprites.draw[draw_index];
    }
    realloc_sprites(min_size) {
        const ctx = this.ctx;
        const old_buffer = this.sprites.attributes;
        this.sprites.attributes = ctx.createBuffer();
        this.sprites.attributes_capacity_bytes = min_size + BASE_SPRITES_CAPACITY;
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.sprites.attributes);
        ctx.bufferData(ctx.ARRAY_BUFFER, this.sprites.attributes_capacity_bytes, ctx.DYNAMIC_DRAW);
        ctx.deleteBuffer(old_buffer);
    }
    update_sprites(updates, message) {
        const ctx = this.ctx;
        const offset = message.offset_bytes();
        const size = message.size_bytes();
        ctx.bindVertexArray(null);
        if (size > this.sprites.attributes_capacity_bytes) {
            this.realloc_sprites(size);
        }
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.sprites.attributes);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, updates.get_data(offset, size));
    }
    build_sprite_vao(instance_base) {
        const GPU_SPRITE_SIZE = 36;
        const ctx = this.ctx;
        const vao = this.next_vao();
        const [position, instance_position, instance_texcoord, instance_data] = this.shaders.sprites_attributes;
        const attributes_offset = instance_base * GPU_SPRITE_SIZE;
        ctx.bindVertexArray(vao);
        // Vertex data
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.sprites.index);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.sprites.vertex);
        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, 8, 0);
        // Instance Data
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.sprites.attributes);
        ctx.enableVertexAttribArray(instance_position);
        ctx.vertexAttribPointer(instance_position, 4, ctx.FLOAT, false, GPU_SPRITE_SIZE, attributes_offset);
        ctx.vertexAttribDivisor(instance_position, 1);
        ctx.enableVertexAttribArray(instance_texcoord);
        ctx.vertexAttribPointer(instance_texcoord, 4, ctx.FLOAT, false, GPU_SPRITE_SIZE, attributes_offset + 16);
        ctx.vertexAttribDivisor(instance_texcoord, 1);
        ctx.enableVertexAttribArray(instance_data);
        ctx.vertexAttribIPointer(instance_data, 1, ctx.INT, GPU_SPRITE_SIZE, attributes_offset + 32);
        ctx.vertexAttribDivisor(instance_data, 1);
        ctx.bindVertexArray(null);
        return vao;
    }
    draw_sprites(message) {
        const instance_base = message.instance_base();
        const instance_count = message.instance_count();
        const texture_id = message.texture_id();
        const draw = this.next_sprite_draw();
        draw.instance_count = instance_count;
        draw.texture = this.textures[texture_id];
        draw.vao = this.build_sprite_vao(instance_base);
    }
    realloc_terrain(min_size) {
        const ctx = this.ctx;
        const old_buffer = this.terrain.attributes;
        this.terrain.attributes = ctx.createBuffer();
        this.terrain.attributes_capacity_bytes = min_size + BASE_TERRAIN_CAPACITY;
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.terrain.attributes);
        ctx.bufferData(ctx.ARRAY_BUFFER, this.terrain.attributes_capacity_bytes, ctx.STATIC_DRAW);
        ctx.deleteBuffer(old_buffer);
    }
    update_terrain(updates, message) {
        const ctx = this.ctx;
        const offset = message.offset_bytes();
        const size = message.size_bytes();
        this.terrain.instance_count = message.cell_count();
        ctx.bindVertexArray(this.terrain.vao);
        if (size > this.terrain.attributes_capacity_bytes) {
            this.realloc_terrain(size);
            this.setup_terrain_vao();
        }
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.terrain.attributes);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, updates.get_data(offset, size));
    }
    realloc_debug(index_size, vertex_size) {
        const ctx = this.ctx;
        const debug = this.debug;
        if (debug.index_capacity < index_size) {
            const old = debug.index;
            debug.index = ctx.createBuffer();
            debug.index_capacity = index_size + BASE_DEBUG_CAPACITY;
            ctx.bindBuffer(ctx.ARRAY_BUFFER, debug.index);
            ctx.bufferData(ctx.ARRAY_BUFFER, debug.index_capacity, ctx.DYNAMIC_DRAW);
            ctx.deleteBuffer(old);
        }
        if (debug.vertex_capacity < vertex_size) {
            const old = debug.vertex;
            debug.vertex = ctx.createBuffer();
            debug.vertex_capacity = vertex_size + BASE_DEBUG_CAPACITY;
            ctx.bindBuffer(ctx.ARRAY_BUFFER, debug.vertex);
            ctx.bufferData(ctx.ARRAY_BUFFER, debug.vertex_capacity, ctx.DYNAMIC_DRAW);
            ctx.deleteBuffer(old);
        }
    }
    draw_debug(updates, message) {
        const ctx = this.ctx;
        const debug = this.debug;
        const index_offset = message.index_offset_bytes();
        const index_size = message.index_size_bytes();
        const vertex_offset = message.vertex_offset_bytes();
        const vertex_size = message.vertex_size_bytes();
        debug.count = message.count();
        ctx.bindVertexArray(this.debug.vao);
        if (debug.index_capacity < index_size || debug.vertex_capacity < vertex_size) {
            this.realloc_debug(index_size, vertex_size);
            this.setup_debug_vao();
        }
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.debug.index);
        ctx.bufferSubData(ctx.ELEMENT_ARRAY_BUFFER, 0, updates.get_data(index_offset, index_size));
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.debug.vertex);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, updates.get_data(vertex_offset, vertex_size));
    }
    prepare_updates() {
        this.ctx.bindVertexArray(null);
        this.sprites.draw_count = 0;
        this.debug.count = 0;
        this.vao_pool_next = 0;
    }
    update(game) {
        this.prepare_updates();
        const updates = game.updates();
        const messages_count = updates.messages_count;
        for (let i = 0; i < messages_count; i += 1) {
            const message = updates.get_message(i);
            const message_name = message.name();
            switch (message_name) {
                case "UpdateSprites": {
                    this.update_sprites(updates, message.update_sprites());
                    break;
                }
                case "DrawSprites": {
                    this.draw_sprites(message.draw_sprites());
                    break;
                }
                case "UpdateTerrain": {
                    this.update_terrain(updates, message.update_terrain());
                    break;
                }
                case "DrawDebug": {
                    this.draw_debug(updates, message.draw_debug());
                    break;
                }
                default: {
                    console.log(`Warning: A drawing update with an unknown type ${message_name} was received`);
                }
            }
        }
    }
    //
    // Render
    //
    render_terrain() {
        const SPRITE_INDEX_COUNT = 6;
        const ctx = this.ctx;
        ctx.useProgram(this.shaders.terrain);
        ctx.activeTexture(ctx.TEXTURE0);
        ctx.bindTexture(ctx.TEXTURE_2D, this.terrain.texture);
        ctx.bindVertexArray(this.terrain.vao);
        ctx.drawElementsInstanced(ctx.TRIANGLES, SPRITE_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0, this.terrain.instance_count);
    }
    render_sprites() {
        const SPRITE_INDEX_COUNT = 6;
        const ctx = this.ctx;
        const sprites = this.sprites;
        ctx.useProgram(this.shaders.sprites);
        ctx.activeTexture(ctx.TEXTURE0);
        for (let i = 0; i < sprites.draw_count; i += 1) {
            const draw = sprites.draw[i];
            ctx.bindTexture(ctx.TEXTURE_2D, draw.texture);
            ctx.bindVertexArray(draw.vao);
            ctx.drawElementsInstanced(ctx.TRIANGLES, SPRITE_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0, draw.instance_count);
        }
    }
    render_debug() {
        const ctx = this.ctx;
        const debug = this.debug;
        if (debug.count > 0) {
            ctx.useProgram(this.shaders.debug);
            ctx.bindVertexArray(debug.vao);
            ctx.drawElementsInstanced(ctx.TRIANGLES, debug.count, ctx.UNSIGNED_SHORT, 0, 1);
        }
    }
    render() {
        const ctx = this.ctx;
        const canvas = this.canvas;
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 1.0]);
        this.render_terrain();
        this.render_sprites();
        this.render_debug();
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
    setup_base_context() {
        const ctx = this.ctx;
        ctx.disable(ctx.CULL_FACE);
        ctx.enable(ctx.BLEND);
        ctx.blendFunc(ctx.ONE, ctx.ONE_MINUS_SRC_ALPHA);
        ctx.blendEquation(ctx.FUNC_ADD);
    }
    setup_shaders() {
        const ctx = this.ctx;
        const assets = this.assets;
        const shaders = this.shaders;
        const sprites = build_shader(ctx, assets, "sprites", ["in_position", "in_instance_position", "in_instance_texcoord", "in_instance_data"], ["view_position", "view_size"]);
        if (sprites) {
            shaders.sprites = sprites.program;
            shaders.sprites_attributes = sprites.attributes;
            shaders.sprites_uniforms = sprites.uniforms;
        }
        else {
            return false;
        }
        const terrain = build_shader(ctx, assets, "terrain", ["in_position", "in_instance_position", "in_instance_texcoord"], ["view_position", "view_size"]);
        if (terrain) {
            shaders.terrain = terrain.program;
            shaders.terrain_attributes = terrain.attributes;
            shaders.terrain_uniforms = terrain.uniforms;
        }
        else {
            return false;
        }
        const debug = build_shader(ctx, assets, "debug", ["in_position", "in_color"], ["view_position", "view_size"]);
        if (debug) {
            shaders.debug = debug.program;
            shaders.debug_attributes = debug.attributes;
            shaders.debug_uniforms = debug.uniforms;
        }
        else {
            return false;
        }
        return true;
    }
    preload_textures() {
        const to_preload = ["atlas", "terrain"];
        for (let name of to_preload) {
            const texture = this.assets.textures.get(name);
            if (!texture) {
                set_last_error(`Failed to preload texture ${name}: missing texture in assets`);
                return false;
            }
            this.textures[texture.id] = create_texture_rgba(this.ctx, texture);
        }
        return true;
    }
    setup_pools() {
        const ctx = this.ctx;
        for (let i = 0; i < 8; i += 1) {
            this.vao_pool.push(ctx.createVertexArray());
            this.sprites.draw.push(new SpritesDraw());
        }
    }
    setup_terrain_vao() {
        const TERRAIN_SPRITE_SIZE = 16;
        const ctx = this.ctx;
        const [position, instance_position, instance_texcoord] = this.shaders.terrain_attributes;
        ctx.bindVertexArray(this.terrain.vao);
        // Vertex data
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.terrain.index);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.terrain.vertex);
        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, 8, 0);
        // Instance Data
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.terrain.attributes);
        ctx.enableVertexAttribArray(instance_position);
        ctx.vertexAttribPointer(instance_position, 2, ctx.FLOAT, false, TERRAIN_SPRITE_SIZE, 0);
        ctx.vertexAttribDivisor(instance_position, 1);
        ctx.enableVertexAttribArray(instance_texcoord);
        ctx.vertexAttribPointer(instance_texcoord, 2, ctx.FLOAT, false, TERRAIN_SPRITE_SIZE, 8);
        ctx.vertexAttribDivisor(instance_texcoord, 1);
        ctx.bindVertexArray(null);
    }
    setup_terrain() {
        const ctx = this.ctx;
        const terrain = this.terrain;
        terrain.index = ctx.createBuffer();
        terrain.vertex = ctx.createBuffer();
        terrain.attributes = ctx.createBuffer();
        terrain.attributes_capacity_bytes = BASE_TERRAIN_CAPACITY;
        terrain.attributes_size_bytes = 0;
        terrain.instance_count = 0;
        terrain.vao = this.vao_pool.pop();
        const texture_id = this.assets.textures.get("terrain")?.id; // Check is handled in preload_textures
        terrain.texture = this.textures[texture_id];
        ctx.bindVertexArray(terrain.vao);
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, terrain.index);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 3, 2, 1, 0, 3]), ctx.STATIC_DRAW);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, terrain.vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, new Float32Array([
            0.0, 0.0, // V0
            1.0, 0.0, // V1
            0.0, 1.0, // V2
            1.0, 1.0, // V3
        ]), ctx.STATIC_DRAW);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, terrain.attributes);
        ctx.bufferData(ctx.ARRAY_BUFFER, terrain.attributes_capacity_bytes, ctx.STATIC_DRAW);
        this.setup_terrain_vao();
    }
    setup_sprites() {
        const ctx = this.ctx;
        const sprites = this.sprites;
        sprites.index = ctx.createBuffer();
        sprites.vertex = ctx.createBuffer();
        sprites.attributes = ctx.createBuffer();
        sprites.attributes_capacity_bytes = BASE_SPRITES_CAPACITY;
        sprites.attributes_size_bytes = 0;
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, sprites.index);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 3, 2, 1, 0, 3]), ctx.STATIC_DRAW);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, sprites.vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, new Float32Array([
            0.0, 0.0, // V0
            1.0, 0.0, // V1
            0.0, 1.0, // V2
            1.0, 1.0, // V3
        ]), ctx.STATIC_DRAW);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, sprites.attributes);
        ctx.bufferData(ctx.ARRAY_BUFFER, sprites.attributes_capacity_bytes, ctx.DYNAMIC_DRAW);
    }
    setup_debug_vao() {
        const DEBUG_VERTEX_SIZE = 12;
        const ctx = this.ctx;
        const [position, color] = this.shaders.debug_attributes;
        ctx.bindVertexArray(this.debug.vao);
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.debug.index);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.debug.vertex);
        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, DEBUG_VERTEX_SIZE, 0);
        ctx.enableVertexAttribArray(color);
        ctx.vertexAttribPointer(color, 4, ctx.UNSIGNED_BYTE, true, DEBUG_VERTEX_SIZE, 8);
        ctx.bindVertexArray(null);
    }
    setup_debug() {
        const ctx = this.ctx;
        const debug = this.debug;
        debug.index = ctx.createBuffer();
        debug.index_capacity = BASE_DEBUG_CAPACITY;
        debug.vertex = ctx.createBuffer();
        debug.vertex_capacity = (BASE_DEBUG_CAPACITY * 1.5) | 0;
        debug.vao = this.vao_pool.pop();
        // Vertex
        ctx.bindVertexArray(debug.vao);
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, debug.index);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, debug.index_capacity, ctx.DYNAMIC_DRAW);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, debug.vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, debug.vertex_capacity, ctx.DYNAMIC_DRAW);
        console.log(debug);
        // Vao
        this.setup_debug_vao();
    }
    setup_uniforms() {
        const ctx = this.ctx;
        const position = new Float32Array([0.0, 0.0]);
        const size = new Float32Array([this.canvas.width, this.canvas.height]);
        let [view_position, view_size] = this.shaders.sprites_uniforms;
        ctx.useProgram(this.shaders.sprites);
        ctx.uniform2fv(view_position, position);
        ctx.uniform2fv(view_size, size);
        [view_position, view_size] = this.shaders.terrain_uniforms;
        ctx.useProgram(this.shaders.terrain);
        ctx.uniform2fv(view_position, position);
        ctx.uniform2fv(view_size, size);
        [view_position, view_size] = this.shaders.debug_uniforms;
        ctx.useProgram(this.shaders.debug);
        ctx.uniform2fv(view_position, position);
        ctx.uniform2fv(view_size, size);
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
//
// Init
//
async function init() {
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
    if (!app.game.start(app.assets)) {
        return null;
    }
    app.ws.open();
    return app;
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
    engine.renderer.handle_resize();
}
/// Execute the game logic of the client for the current frame
function game_updates(engine, time) {
    engine.game.instance.update(time);
}
/// Reads the rendering updates generated by the game client
function renderer_updates(engine) {
    engine.renderer.update(engine.game);
}
function update(engine, time) {
    websocket_messages(engine);
    handle_resize(engine);
    game_updates(engine, time);
    renderer_updates(engine);
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
    const engine = await init();
    if (!engine) {
        console.log("Failed to initialize application");
        return;
    }
    boundedRun = run.bind(null, engine);
    boundedRun();
}
init_app();
