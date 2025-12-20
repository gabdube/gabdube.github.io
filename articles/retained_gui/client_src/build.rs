use std::fs;
use std::fmt::Write;
use std::mem::offset_of;

#[allow(dead_code)]
enum ArgType {
    Str(&'static str),
    Usize(usize)
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UpdateGuiMessageParams {
    pub index_bytes_offset: u32,
    pub index_bytes_size: u32,
    pub vertex_bytes_offset: u32,
    pub vertex_bytes_size: u32
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawGuiMessageParams {
    pub draw_count: u32,
    pub index_bytes_offset: u32,
    pub vertex_bytes_offset: u32,
    pub image_texture: u32,
    pub font_texture: u32,
    pub scissor: [u16; 4],
}

// Note: This is a union!
#[repr(C)]
#[derive(Copy, Clone)]
pub union OutputMessageParams {
    pub update_gui: UpdateGuiMessageParams,
    pub draw_gui: DrawGuiMessageParams,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum OutputMessageType {
    ClearGui=1,
    UpdateGui,
    DrawGui,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OutputMessage {
    pub ty: OutputMessageType,
    pub params: OutputMessageParams
}


#[repr(C)]
pub struct OutputIndex {
    pub messages_count: u32,
    pub messages_size: u32,
    pub messages_ptr: u32,
    pub data_ptr: u32,
}

fn write_helpers(output: &mut String) {
    output.push_str(
"
function getUint16Array(data: DataView, offset: number, count: number): number[] {
  const values = [];
  for(let x=0; x<count; x++) { values.push(data.getUint16(offset + (2*x), true)); }
  return values;
}
");

    output.push_str("\n");
}

fn write_message_types(output: &mut String) {
    let all = [OutputMessageType::ClearGui, OutputMessageType::UpdateGui, OutputMessageType::DrawGui];
    for value in all {
        let line = format!("export const {:?} = {};\n", value, value as u32);
        output.push_str(&line);
    }

    let mut line = "type GameMessageType =".to_string();
    for value in all {
        line.push_str(&format!(" {} |", value as u32));
    };
    output.push_str(&line[0..line.len()-2]);

    output.push_str(";\n\n");
}

/// A struct that contain only primitive
fn generate_struct(
    out: &mut String,
    name: &str,
    fields: &[(&str, &str, usize)],
    complex_fields: &[(&str, &str, &[ArgType])],
) {
    write!(out, "export class {name} {{\n").unwrap();
    write!(out, "  data: DataView;\n").unwrap();
    write!(out, "  constructor(data: DataView) {{\n    this.data = data;\n  }}\n").unwrap();

    for (field_name, accessor, offset) in fields.iter() {
        write!(out, "  {field_name}() {{ return this.data.{accessor}({offset}, true); }}\n").unwrap();
    }

    for (field_name, function, args) in complex_fields.iter() {
        let mut args_formatted = String::with_capacity(32);
        for arg in args.iter() {
            match arg {
                ArgType::Str(value) => { args_formatted.push_str(value); }
                ArgType::Usize(value) => { args_formatted.push_str(&value.to_string()); }
            }
            args_formatted.push_str(", ");
        }

        write!(out, "  {field_name}() {{ return {function}(this.data, {args_formatted}); }}\n").unwrap();
    }

    write!(out, "}}\n").unwrap();
}

fn write_game_message_api(output: &mut String) {
    let code = [
    "export class GameUpdateMessage {",
    "  ty: GameMessageType;",
    "  params: DataView;",
    "  constructor(ty: GameMessageType, params: DataView) {",
    "    this.ty = ty;",
    "    this.params = params;",
    "  }",
    ];

    for line in code {
        output.push_str(line);
        output.push('\n');
    }

    
    let methods = [
        ["update_gui", "UpdateGuiMessageParams"],
        ["draw_gui", "DrawGuiMessageParams"],
    ];
    for [m1, m2] in methods {
        output.push_str(&format!("  {m1}() {{\n"));
        output.push_str(&format!("    return new {m2}(this.params);\n"));
        output.push_str("  }\n");   
    }

    output.push_str("}\n");
}

fn write_get_message_function() -> String {
    let message_size = size_of::<OutputMessage>();
    let message_ty_size = size_of::<OutputMessageType>();
    let message_data_size = size_of::<OutputMessageParams>();
    let line1 = format!("    const message_ptr = this.messages_ptr + (index * {message_size});");
    let line2 = format!("    const ty = new DataView(this.buffer, message_ptr, {message_ty_size}).getUint32(0, true);");
    let line3 = format!("    if (ty < {:?} || ty > {:?}) {{", OutputMessageType::ClearGui, OutputMessageType::DrawGui);
    let line4 = format!("    const params = new DataView(this.buffer, message_ptr+{message_ty_size}, {message_data_size})");
    let code = [
        "    if (index >= this.messages_count) {",
        "      console.error(`Tried to read message beyond total message count ${index} >= ${this.messages_count}`);",
        "      return null;",
        "    }",
        line1.as_str(),
        line2.as_str(),
        line3.as_str(),
        "      console.error(`Received unknown message type: ${ty}`);",
        "      return null;",
        "    }",
        line4.as_str(),
        "    return new GameUpdateMessage(ty as GameMessageType, params);",
    ];

    let mut output = String::with_capacity(100);
    for line in code {
        output.push_str(line);
        output.push('\n');
    }

    output
}

fn write_game_updates_api(output: &mut String) {
    let index_size = size_of::<OutputIndex>();
    let line1 = format!("    const index = new GameUpdateIndex(new DataView(buffer, output_index_ptr, {index_size}));");
    let line2 = write_get_message_function();
    let code = [
        "export class GameUpdatesApi {",
        "  buffer: ArrayBuffer;",
        "  messages_count: number;",
        "  messages_size: number;",
        "  messages_ptr: number;",
        "  data_ptr: number;",
        "  constructor(buffer: ArrayBuffer, output_index_ptr: number) {",
        line1.as_str(),
        "    this.buffer = buffer;",
        "    this.messages_count = index.messages_count();",
        "    this.messages_size = index.messages_size();",
        "    this.messages_ptr = index.messages_ptr();",
        "    this.data_ptr = index.data_ptr();",
        "  }",
        "  get_message(index: number): GameUpdateMessage | null {",
        line2.as_str(),
        "  }",
        "  get_data(offset: number, size: number): Uint8Array {",
        "    return new Uint8Array(this.buffer, this.data_ptr+offset, size);",
        "  }",
        "}"
    ];

    for line in code {
        output.push_str(line);
        output.push('\n');
    }
}

fn write_api_file(output: String) -> bool {
    if let Err(_e) = fs::remove_file("../engine/game_interface_api.ts") {}
    fs::write("../engine/game_interface_api.ts", output).is_ok()
}

fn main() {
    // This is always be "Uint32" with wasm
    let pointer_type = "getUint32";

    let mut output = String::with_capacity(2000);
    output.push_str("/// This file was auto-generated\n\n");

    write_helpers(&mut output);
    write_message_types(&mut output);

    generate_struct(
        &mut output,
        "UpdateGuiMessageParams",
        &[
            ("index_bytes_offset", "getUint32", offset_of!(UpdateGuiMessageParams, index_bytes_offset)),
            ("index_bytes_size", "getUint32", offset_of!(UpdateGuiMessageParams, index_bytes_size)),
            ("vertex_bytes_offset", "getUint32", offset_of!(UpdateGuiMessageParams, vertex_bytes_offset)),
            ("vertex_bytes_size", "getUint32", offset_of!(UpdateGuiMessageParams, vertex_bytes_size)),
        ],
        &[]
    );

    generate_struct(
        &mut output,
        "DrawGuiMessageParams",
        &[
            ("draw_count", "getUint32", offset_of!(DrawGuiMessageParams, draw_count)),
            ("index_bytes_offset", "getUint32", offset_of!(DrawGuiMessageParams, index_bytes_offset)),
            ("vertex_bytes_offset", "getUint32", offset_of!(DrawGuiMessageParams, vertex_bytes_offset)),
            ("image_texture", "getUint32", offset_of!(DrawGuiMessageParams, image_texture)),
            ("font_texture", "getUint32", offset_of!(DrawGuiMessageParams, font_texture)),
            
        ],
        &[
            ("scissor", "getUint16Array", &[ArgType::Usize(offset_of!(DrawGuiMessageParams, scissor)), ArgType::Usize(4)]),
        ],
    );

    generate_struct(
        &mut output,
        "GameUpdateIndex",
        &[
            ("messages_count", pointer_type, offset_of!(OutputIndex, messages_count)),
            ("messages_size", pointer_type, offset_of!(OutputIndex, messages_size)),
            ("messages_ptr", pointer_type, offset_of!(OutputIndex, messages_ptr)),
            ("data_ptr", pointer_type, offset_of!(OutputIndex, data_ptr)),
        ],
        &[]
    );

    write_game_message_api(&mut output);
    write_game_updates_api(&mut output);

    if !write_api_file(output) {
        eprintln!("Failed to write api file");
    }
}