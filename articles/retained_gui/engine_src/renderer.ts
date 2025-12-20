import { set_last_error } from "./error";
import { GameInterface, GameUpdates, ClearGui, UpdateGui, DrawGui } from "./game_interface";
import type { UpdateGuiMessageParams, DrawGuiMessageParams } from "./game_interface";
import type { EngineAssets, Texture } from "./assets";

const NO_TEXTURE: number = 4_294_967_295;
const GUI_INDEX_BLOCK_SIZE: number = 2048;
const GUI_VERTEX_BLOCK_SIZE: number = 2048 * 10;

class RendererCanvas {
    container: HTMLElement;
    element: HTMLCanvasElement;
    width: number;
    height: number;

    constructor(container: HTMLCanvasElement, element: HTMLCanvasElement) {
        this.container = container;
        this.element = element;
        this.width = 0;
        this.height = 0;
    }
}

class RendererShaders {
    gui_attributes: number[] = [];  // position, texcoord, color
    gui_uniforms: WebGLUniformLocation[] = [];  // View position, View size
    gui: WebGLProgram | null = null;
}

class GuiDrawCommand {
    font_texture: WebGLTexture;
    image_texture: WebGLTexture;
    vao: WebGLVertexArrayObject;
    draw_count: number;
    index_bytes_offset: number;
    vertex_bytes_offset: number;
    scissor: number[];

    constructor(
        vao: WebGLVertexArrayObject,
        draw_count: number,
        index_bytes_offset: number,
        vertex_bytes_offset: number,
        font_texture: WebGLTexture,
        image_texture: WebGLTexture,
        scissor: number[],
    ) {
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
    index: WebGLBuffer | null = null;
    index_capacity: number = 0;
    index_size: number = 0;

    vertex: WebGLBuffer | null = null;
    vertex_capacity: number = 0;
    vertex_size: number = 0;

    draw_commands_count: number = 0;
    draw_commands: GuiDrawCommand[] = [];
}

export class Renderer {
    _canvas: RendererCanvas | null = null;
    _ctx: WebGL2RenderingContext | null = null;
    _framebuffer: WebGLFramebuffer | null = null;
    color: WebGLRenderbuffer | null = null;
    visible: boolean = false;

    _assets: EngineAssets | null = null;
    textures: (WebGLTexture|null)[] = [];
    default_texture: WebGLTexture | null = null;

    shaders: RendererShaders = new RendererShaders();
    gui: RendererGui = new RendererGui();

    vao_pool: WebGLVertexArrayObject[] = [];
    global_vao: WebGLVertexArrayObject | null = null;

    init(): boolean {
        if ( !this.setup_canvas() ) { return false };
        if ( !this.setup_context() ) { return false; }
        if ( !this.setup_framebuffer() ) { return false; }
        this.setup_base_context();
        this.setup_vao_pool();  

        return true;
    }

    refresh() {
        this.handle_resize();
    }

    init_default_resources(assets: EngineAssets): boolean {
        this._assets = assets;

        if (!this.setup_shaders()) { return false; };
        if (!this.preload_textures()) { return false; };

        this.setup_gui();
        this.setup_uniforms();

        this.visible = true;

        return true;
    }

    canvas(): RendererCanvas {
        if (!this._canvas) { throw "Canvas not initialized"; }
        return this._canvas;
    }

    ctx(): WebGL2RenderingContext {
        if (!this._ctx) { throw "Context not initialized"; }
        return this._ctx;
    }

    framebuffer(): WebGLFramebuffer {
        if (!this._framebuffer) { throw "Framebuffer not initialized"; }
        return this._framebuffer;
    }

    assets(): EngineAssets {
        if (!this._assets) { throw "Engine assets not set"; }
        return this._assets;
    }

    //
    // Resize
    //

    private handle_resize_framebuffer(): boolean {
        const canvas = this.canvas();
        const display_width  = canvas.container.clientWidth;
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

    private handle_resize_uniforms() {
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

    handle_resize(): boolean {
        
        if (!this.handle_resize_framebuffer()) {
            return false;
        }
    
        this.handle_resize_uniforms();

        return true;
    }

    //
    // Update
    //

    private clear_gui() {
        const gui = this.gui;
        for (let i=0; i<gui.draw_commands_count; i++) {
            this.vao_pool.push(gui.draw_commands[i].vao);
        }
        gui.draw_commands_count = 0;
    }

    private update_gui(updates: GameUpdates, params: UpdateGuiMessageParams) {
        const ctx = this.ctx();
        const gui = this.gui;
        const index_offset = params.index_bytes_offset();
        const index_size = params.index_bytes_size();
        const vertex_offset = params.vertex_bytes_offset();
        const vertex_size = params.vertex_bytes_size();

        // Realloc
        if (index_size > gui.index_capacity && gui.index) {
            const new_capacity = index_size + GUI_INDEX_BLOCK_SIZE;
            gui.index = realloc_buffer(ctx, gui.index, ctx.ELEMENT_ARRAY_BUFFER, gui.index_capacity, new_capacity, true);
            gui.index_capacity = new_capacity;
        }

        if (vertex_size > gui.vertex_capacity && gui.vertex) {
            const new_capacity = vertex_size + GUI_VERTEX_BLOCK_SIZE;
            gui.vertex = realloc_buffer(ctx, gui.vertex, ctx.ARRAY_BUFFER, gui.vertex_capacity, new_capacity, true);
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
        for (let i=0; i<gui.draw_commands_count; i++) {
            const cmd = gui.draw_commands[i];
            this.write_gui_vao(cmd.vao, cmd.vertex_bytes_offset);
        }
    }

    private draw_gui(params: DrawGuiMessageParams) {
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

    private prepare_updates() {
        this.ctx().bindVertexArray(null);
    }

    update(game: GameInterface) { 
        this.prepare_updates();

        const updates = game.updates();
        const message_count = updates.message_count();
        for (let i=0; i<message_count; i++) {
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

    private render_gui() {
        const ctx = this.ctx();
        const gui = this.gui;

        if (gui.draw_commands_count == 0 || !this.shaders.gui) {
            return;
        }

        ctx.enable(ctx.SCISSOR_TEST);

        ctx.useProgram(this.shaders.gui);

        for (let i=0; i<gui.draw_commands_count; i++) {
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

    render(time: DOMHighResTimeStamp) {
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

    private setup_base_context() {
        const ctx = this.ctx();
        ctx.disable(ctx.CULL_FACE);
        ctx.enable(ctx.BLEND);
        ctx.blendFunc(ctx.ONE, ctx.ONE_MINUS_SRC_ALPHA);
        ctx.blendEquation(ctx.FUNC_ADD);
    }

    private setup_canvas(): boolean {
        const demo = document.getElementById("demo") as HTMLCanvasElement;
        const canvas_elem = document.getElementById("canvas") as HTMLCanvasElement;
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

    private setup_context(): boolean {
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

    private setup_framebuffer(): boolean {
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

    private setup_shaders(): boolean {
        const ctx = this.ctx();
        const assets = this.assets();
        const shaders = this.shaders;

        const gui = build_shader(ctx, assets, "gui",
            ["in_position", "in_texcoord", "in_color", "in_data"],
            ["view_size", "image_texture", "font_texture"]
        );
        if (gui) {
            shaders.gui = gui.program;
            shaders.gui_attributes = gui.attributes;
            shaders.gui_uniforms = gui.uniforms;
        } else {
            return false;
        }

        return true;
    }

    private setup_vao_pool() {
        const ctx = this.ctx();
        for (let i=0; i<16; i++) {
            this.vao_pool.push(ctx.createVertexArray());
        }

        this.global_vao = ctx.createVertexArray();
    }

    private preload_textures(): boolean {
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

    private setup_gui() {
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

    private setup_uniforms() {
        const ctx = this.ctx();
        const canvas = this.canvas();
        const shaders = this.shaders;
        const size = new Float32Array([canvas.width, canvas.height]);

        ctx.useProgram(shaders.gui);
        ctx.uniform2fv(shaders.gui_uniforms[0], size); // view_size
        ctx.uniform1i(shaders.gui_uniforms[1], 0); // image_texture
        ctx.uniform1i(shaders.gui_uniforms[2], 1); // font_texture
    }

    private get_samples(): number {
        let max_samples = this.ctx().getParameter(this.ctx().MAX_SAMPLES);

        function is_mobile() {
            let check = false;
            (function(a){if(/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino|android|ipad|playbook|silk/i.test(a)||/1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(a.substr(0,4))) check = true;})(navigator.userAgent||navigator.vendor||(window as any).opera);
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

        return max_samples
    }

    private write_gui_vao(vao: WebGLVertexArrayObject, vertex_bytes_offset: number) {
        const GUI_VERTEX_SIZE = 24;

        const ctx = this.ctx();
        const gui = this.gui;
        const [position, texcoord, color, data] = this.shaders.gui_attributes;

        ctx.bindVertexArray(vao);
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, gui.index)
        ctx.bindBuffer(ctx.ARRAY_BUFFER, gui.vertex)

        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, GUI_VERTEX_SIZE, vertex_bytes_offset+0);

        ctx.enableVertexAttribArray(texcoord);
        ctx.vertexAttribPointer(texcoord, 2, ctx.FLOAT, false, GUI_VERTEX_SIZE, vertex_bytes_offset+8);

        ctx.enableVertexAttribArray(color);
        ctx.vertexAttribPointer(color, 4, ctx.UNSIGNED_BYTE, true, GUI_VERTEX_SIZE, vertex_bytes_offset+16);

        ctx.enableVertexAttribArray(data);
        ctx.vertexAttribIPointer(data, 1, ctx.UNSIGNED_INT, GUI_VERTEX_SIZE, vertex_bytes_offset+20);

        ctx.bindVertexArray(null);
    }

    //
    // Helpers
    //

    private get_texture(texture_id: number): WebGLTexture {
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

    private create_texture(texture_id: number): WebGLTexture {
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
        return texture
    }
}

function build_shader(
    ctx: WebGL2RenderingContext,
    assets: EngineAssets,
    shader_name: string,
    attributes_names: string[],
    uniforms_names: string[]
): {program: WebGLProgram, attributes: number[], uniforms: WebGLUniformLocation[]} | undefined 
{
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

    const attributes: number[] = []
    for (let attribute_name of attributes_names) {
        const loc = ctx.getAttribLocation(program, attribute_name);
        if (loc == -1) {
            set_last_error(`Unkown attribute "${attribute_name}" in shader "${shader_name}"`);
            return
        }

        attributes.push(loc);
    }

    const uniforms: WebGLUniformLocation[] = [];
    for (let uniform_name of uniforms_names) {
        const loc = ctx.getUniformLocation(program, uniform_name) as any;
        if (!loc) {
            set_last_error(`Unkown uniform "${uniform_name}" in shader "${shader_name}"`);
            return
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

function create_shader(ctx: WebGL2RenderingContext, type: GLenum, source: string): WebGLShader|undefined {
    const shader = ctx.createShader(type) as WebGLShader;
    ctx.shaderSource(shader, source);
    ctx.compileShader(shader);
    const success = ctx.getShaderParameter(shader, ctx.COMPILE_STATUS);
    if (success) {
        return shader;
    }

    console.log(ctx.getShaderInfoLog(shader));
    ctx.deleteShader(shader);
}

function create_program(ctx: WebGL2RenderingContext, vertexShader: WebGLShader, fragmentShader: WebGLShader): WebGLProgram|undefined {
    const program = ctx.createProgram() as WebGLProgram;
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

function create_default_texture(ctx: WebGL2RenderingContext): WebGLTexture {
    const dimension = 4;
    const pixel_size = 4;
    const byte_size = dimension * dimension * pixel_size;
    const data = new Uint8Array(byte_size);
    for (let i=0; i<byte_size; i+=4) {
        data[i+0] = 255;
        data[i+1] = 0;
        data[i+2] = 255;
        data[i+3] = 255;
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

function create_texture_rgba(ctx: WebGL2RenderingContext, cpu_texture: Texture): WebGLTexture {
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

function realloc_buffer(
    ctx: WebGL2RenderingContext,
    buffer: WebGLBuffer,
    target: GLenum,
    old_capacity: number,
    new_capacity: number,
    copy_data: boolean
): WebGLBuffer {
    const new_buffer = ctx.createBuffer();
    ctx.bindBuffer(target, new_buffer);
    ctx.bufferData(target, new_capacity, ctx.DYNAMIC_DRAW);

    if (copy_data) {
        ctx.bindBuffer(ctx.COPY_READ_BUFFER, buffer);
        ctx.bindBuffer(ctx.COPY_WRITE_BUFFER, new_buffer);
        ctx.copyBufferSubData(ctx.COPY_READ_BUFFER, ctx.COPY_WRITE_BUFFER, 0, 0, old_capacity);
        ctx.bindBuffer(ctx.COPY_READ_BUFFER, null);
        ctx.bindBuffer(ctx.COPY_WRITE_BUFFER, null);
    }

    ctx.deleteBuffer(buffer);

    return new_buffer;
}

function next_vao(ctx: WebGL2RenderingContext, pool: WebGLVertexArrayObject[]): WebGLVertexArrayObject {
    if (pool.length === 0) {
        for (let i=0; i<16; i++) {
            pool.push(ctx.createVertexArray());
        }
    }

    return pool.pop() as WebGLVertexArrayObject;
}

