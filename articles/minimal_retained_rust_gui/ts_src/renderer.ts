import { set_last_error } from "./error";
import { GameInterface, GameUpdates } from "./game_interface";
import { EngineAssets, Texture } from "./assets";

const BASE_TERRAIN_CAPACITY = 1024 * 10;

class RendererCanvas {
    container: HTMLElement;
    element: HTMLCanvasElement;
    width: number;
    height: number;

    constructor(container: HTMLElement, element: HTMLCanvasElement) {
        this.container = container;
        this.element = element;
        this.width = 0;
        this.height = 0;
    }
}

class RendererShaders {
    terrain_attributes: number[];  // position, instance_position, instance_texcoord
    terrain_uniforms: WebGLUniformLocation[];  // View position, View size
    terrain: WebGLProgram;
}

class Terrain {
    index: WebGLBuffer;
    vertex: WebGLBuffer;
    attributes: WebGLBuffer;
    attributes_size_bytes: number;
    attributes_capacity_bytes: number;
    texture: WebGLTexture;
    instance_count: number;
    vao: WebGLVertexArrayObject;
}

export class Renderer {
    canvas: RendererCanvas;
    ctx: WebGL2RenderingContext;
    framebuffer: WebGLFramebuffer;
    color: WebGLRenderbuffer;
    visible: boolean = false;

    assets: EngineAssets;

    shaders: RendererShaders = new RendererShaders();
    textures: WebGLTexture[] = [];

    terrain: Terrain = new Terrain();

    init(): boolean {
        if ( !this.setup_canvas() ) { return false };
        if ( !this.setup_context() ) { return false; }
        if ( !this.setup_framebuffer() ) { return false; }

        return true;
    }

    init_default_resources(assets: EngineAssets): boolean {
        this.assets = assets;

        if (!this.setup_shaders()) { return false; };
        if (!this.preload_textures()) { return false; }

        this.setup_terrain();
        this.setup_uniforms();

        this.visible = true;

        return true;
    }

    //
    // Resize
    //

    private handle_resize_framebuffer(): boolean {
        const canvas = this.canvas;
        const display_width  = canvas.container.clientWidth;
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

    private handle_resize_uniforms() {
        const ctx = this.ctx;
        const size = new Float32Array([this.canvas.width, this.canvas.height]);
        const size_uniforms = [
            [this.shaders.terrain, this.shaders.terrain_uniforms[1]],
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

    private update_terrain(updates: GameUpdates, message: any) {
        function realloc_terrain(ctx: WebGL2RenderingContext, terrain: Terrain, min_size: number) {
            const new_capacity = min_size + BASE_TERRAIN_CAPACITY;
            terrain.attributes = realloc_buffer(ctx, terrain.attributes, ctx.ARRAY_BUFFER, terrain.attributes_capacity_bytes, new_capacity, false);
            terrain.attributes_capacity_bytes = new_capacity;
        }

        const ctx = this.ctx;

        const offset = message.offset_bytes();
        const size = message.size_bytes();
        this.terrain.instance_count = message.cell_count();

        ctx.bindVertexArray(this.terrain.vao);

        if (size > this.terrain.attributes_capacity_bytes) {
            realloc_terrain(ctx, this.terrain, size)
            this.setup_terrain_vao();
        }
        
        this.terrain.attributes_size_bytes = size;
        
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.terrain.attributes);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, updates.get_data(offset, size));
    }

    private prepare_updates() {
        this.ctx.bindVertexArray(null);
    }

    private update_view_offset(message: [number, number]) {
        const ctx = this.ctx;
        const offset = new Float32Array(message); // message is the [x, y] view offset
        const offset_uniforms: [WebGLProgram, WebGLUniformLocation][] = [
            [this.shaders.terrain, this.shaders.terrain_uniforms[0]],
        ];

        for (let [shader, uniform] of offset_uniforms) {
            ctx.useProgram(shader);
            ctx.uniform2fv(uniform, offset);
        }
    }

    private update_view_size(message: [number, number]) {
        const ctx = this.ctx;
        const size = new Float32Array(message); // message is the [width, height] view size
        const size_uniforms: [WebGLProgram, WebGLUniformLocation][] = [
            [this.shaders.terrain, this.shaders.terrain_uniforms[1]],
        ];

        for (let [shader, uniform] of size_uniforms) {
            ctx.useProgram(shader);
            ctx.uniform2fv(uniform, size);
        }
    }

    update(game: GameInterface) { 
        this.prepare_updates();

        const updates = game.updates();
        const messages_count = updates.messages_count;
        for (let i = 0; i < messages_count; i += 1) {
            const message = updates.get_message(i);
            const message_name = message.name();
            switch (message_name) {
                case "UpdateTerrain": {
                    this.update_terrain(updates, message.update_terrain())
                    break;
                }
                case "UpdateViewOffset": {
                    this.update_view_offset(message.update_view_offset());
                    break;
                }
                case "UpdateViewSize": {
                    this.update_view_size(message.update_view_size());
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

    private render_terrain() {
        const SPRITE_INDEX_COUNT: number = 6;
        const ctx = this.ctx;

        if (this.terrain.instance_count > 0) {
            ctx.useProgram(this.shaders.terrain);
            ctx.activeTexture(ctx.TEXTURE0);

            ctx.bindTexture(ctx.TEXTURE_2D, this.terrain.texture);
            ctx.bindVertexArray(this.terrain.vao);
            ctx.drawElementsInstanced(ctx.TRIANGLES, SPRITE_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0, this.terrain.instance_count);
        }
    }

    render() {
        const ctx = this.ctx;
        const canvas = this.canvas;

        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 1.0]);

        this.render_terrain();

        ctx.bindFramebuffer(ctx.READ_FRAMEBUFFER, this.framebuffer);
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, null);
        ctx.blitFramebuffer(0, 0, canvas.width, canvas.height, 0, 0, canvas.width, canvas.height, ctx.COLOR_BUFFER_BIT, ctx.LINEAR);
    }

    //
    // Setup
    //

    private setup_canvas(): boolean {
        const demo = document.getElementById("demo") as HTMLCanvasElement;
        const canvas_elem = document.getElementById("canvas") as HTMLCanvasElement;
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

    private setup_context(): boolean {
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

    private setup_framebuffer(): boolean {
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

    private get_samples(): number {
        let max_samples = this.ctx.getParameter(this.ctx.MAX_SAMPLES);

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

    private setup_shaders(): boolean {
        const ctx = this.ctx;
        const assets = this.assets;
        const shaders = this.shaders;

        const terrain = build_shader(ctx, assets, "terrain",
            ["in_position", "in_instance_position", "in_instance_texcoord"],
            ["view_position", "view_size"]
        );
        if (terrain) {
            shaders.terrain = terrain.program;
            shaders.terrain_attributes = terrain.attributes;
            shaders.terrain_uniforms = terrain.uniforms;
        } else {
            return false;
        }

        return true;
    }

    private preload_textures(): boolean {
        const to_preload = ["terrain"];

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

    private setup_terrain_vao() {
        const TERRAIN_VERTEX_SIZE = 8;
        const TERRAIN_SPRITE_SIZE = 16;
        const ctx = this.ctx;
        const [position, instance_position, instance_texcoord] = this.shaders.terrain_attributes;

        ctx.bindVertexArray(this.terrain.vao);

        // Vertex data
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.terrain.index);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.terrain.vertex);
        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, TERRAIN_VERTEX_SIZE, 0);

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

    private setup_terrain() {
        const ctx = this.ctx;
        const terrain = this.terrain;
        
        terrain.index = ctx.createBuffer();
        terrain.vertex = ctx.createBuffer();
        terrain.attributes = ctx.createBuffer();
        terrain.attributes_capacity_bytes = BASE_TERRAIN_CAPACITY;
        terrain.attributes_size_bytes = 0;
        terrain.instance_count = 0;

        terrain.vao = ctx.createVertexArray();

        const texture_id = this.assets.textures.get("terrain")?.id as number;  // Check is handled in preload_textures
        terrain.texture = this.textures[texture_id];

        ctx.bindVertexArray(terrain.vao);

        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, terrain.index);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 3, 2, 1, 0, 3]), ctx.STATIC_DRAW);

        ctx.bindBuffer(ctx.ARRAY_BUFFER, terrain.vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER,  new Float32Array([
            0.0, 0.0, // V0
            1.0, 0.0, // V1
            0.0, 1.0, // V2
            1.0, 1.0, // V3
        ]), ctx.STATIC_DRAW);

        ctx.bindBuffer(ctx.ARRAY_BUFFER, terrain.attributes);
        ctx.bufferData(ctx.ARRAY_BUFFER, terrain.attributes_capacity_bytes, ctx.STATIC_DRAW);

        this.setup_terrain_vao();
    }

    private setup_uniforms() {
        const ctx = this.ctx;
        const position = new Float32Array([0.0, 0.0]);
        const size = new Float32Array([this.canvas.width, this.canvas.height]);

        let [view_position, view_size] = this.shaders.terrain_uniforms;
        ctx.useProgram(this.shaders.terrain);
        ctx.uniform2fv(view_position, position);
        ctx.uniform2fv(view_size, size);
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
