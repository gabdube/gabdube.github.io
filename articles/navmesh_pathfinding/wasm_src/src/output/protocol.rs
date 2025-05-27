//! Dynamically generate a javascript types that will be able parse the data returned by `GameClient::updates_ptr`
//! Note the main logic is still located in `game_interface.ts` and `renderer.ts`

use std::mem::offset_of;
use std::fmt::Write;
use super::message::*;
use super::OutputIndex;

/// Generate a map of "[raw_value]: [string_name]" to map rust enum value to their name.
fn generate_enum<T: Copy + Into<u32>>(
    out: &mut String,
    name: &str,
    fields: &[(&str, T)]
) {
    write!(out, "const {name} = Object.freeze({{\n").unwrap();
    for (name, value) in fields.iter() {
        let raw_value: u32 = (*value).into();
        write!(out, "  {raw_value}: \'{name}\',\n").unwrap();
    }
    write!(out, "}})\n").unwrap();
}

/// A struct that contain only primitive
fn generate_struct(
    out: &mut String,
    name: &str,
    size: usize,
    fields: &[(&str, &str, usize)],
) {
    write!(out, "export class {name} {{\n").unwrap();

    write!(out, r#"
        constructor(buffer, ptr) {{
            this.view = new DataView(buffer, ptr, {size});
        }}
    "#).unwrap();

    for (field_name, accessor, offset) in fields.iter() {
        write!(out, r#"
            {field_name}() {{
                return this.view.{accessor}({offset}, true);
            }}
        "#).unwrap();
    }

    write!(out, "\n}}\n").unwrap();
}

fn generate_struct_with_custom_fields(
    out: &mut String,
    name: &str,
    size: usize,
    fields: &[(&str, &str, usize)],
    custom_fields: &[(&str, &str)]
) {
    write!(out, "export class {name} {{\n").unwrap();

    write!(out, r#"
        constructor(buffer, ptr) {{
            this.view = new DataView(buffer, ptr, {size});
        }}
    "#).unwrap();

    for (field_name, accessor, offset) in fields.iter() {
        write!(out, r#"
            {field_name}() {{
                return this.view.{accessor}({offset}, true);
            }}
        "#).unwrap();
    }

    for (field_name, code) in custom_fields.iter() {
        write!(out,  r#"
            {field_name}() {{ 
                {code}
            }}
        "#).unwrap();
    }

    write!(out, "\n}}\n").unwrap();
}

pub fn compile() -> String {
    let mut source = String::with_capacity(1024);

    // This is always be "Uint32" with wasm
    let pointer_type = match size_of::<usize>() {
        4 => "getUint32",
        8 => "getBigInt64",
        size => { panic!("Unexpected pointer size \"{size}\""); }
    };

    generate_enum(
        &mut source,
        "OutputMessageType",
        &[
            ("UpdateSprites", OutputMessageType::UpdateSprites),
            ("DrawSprites", OutputMessageType::DrawSprites),
            ("UpdateTerrain", OutputMessageType::UpdateTerrain),
            ("DrawDebug", OutputMessageType::DrawDebug),
        ]
    );

    generate_struct(
        &mut source, 
        "OutputIndex", 
        size_of::<OutputIndex>(),
        &[
            ("messages_count", pointer_type, offset_of!(OutputIndex, messages_count)),
            ("messages_size", pointer_type, offset_of!(OutputIndex, messages_size)),
            ("messages_ptr", pointer_type, offset_of!(OutputIndex, messages_ptr)),
            ("data_ptr", pointer_type, offset_of!(OutputIndex, data_ptr)),
        ],
    );

    generate_struct(
        &mut source, 
        "UpdateSpritesParams", 
        size_of::<UpdateSpritesParams>(),
        &[
            ("offset_bytes", pointer_type, offset_of!(UpdateSpritesParams, offset_bytes)),
            ("size_bytes", pointer_type, offset_of!(UpdateSpritesParams, size_bytes)),
        ],
    );

    generate_struct(
        &mut source, 
        "DrawSpritesParams", 
        size_of::<DrawSpritesParams>(),
        &[
            ("instance_base", "getUint32", offset_of!(DrawSpritesParams, instance_base)),
            ("instance_count", "getUint32", offset_of!(DrawSpritesParams, instance_count)),
            ("texture_id", "getUint32", offset_of!(DrawSpritesParams, texture_id)),
        ],
    );

    generate_struct(
        &mut source, 
        "UpdateTerrainParams", 
        size_of::<UpdateTerrainParams>(),
        &[
            ("offset_bytes", pointer_type, offset_of!(UpdateTerrainParams, offset_bytes)),
            ("size_bytes", pointer_type, offset_of!(UpdateTerrainParams, size_bytes)),
            ("cell_count", pointer_type, offset_of!(UpdateTerrainParams, cell_count)),
        ],
    );

    generate_struct(
        &mut source, 
        "DrawDebugParams", 
        size_of::<DrawDebugParams>(),
        &[
            ("index_offset_bytes", pointer_type, offset_of!(DrawDebugParams, index_offset_bytes)),
            ("index_size_bytes", pointer_type, offset_of!(DrawDebugParams, index_size_bytes)),
            ("vertex_offset_bytes", pointer_type, offset_of!(DrawDebugParams, vertex_offset_bytes)),
            ("vertex_size_bytes", pointer_type, offset_of!(DrawDebugParams, vertex_size_bytes)),
            ("count", pointer_type, offset_of!(DrawDebugParams, count)),
        ],
    );

    generate_struct_with_custom_fields(
        &mut source, 
        "OutputMessage", 
        size_of::<OutputMessage>(),
        &[
            ("ty", "getUint32", offset_of!(OutputMessage, ty)),
        ],
        &[
            ("name", "return OutputMessageType[this.ty()] || this.ty();"),
            ("update_sprites", "return new UpdateSpritesParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("draw_sprites", "return new DrawSpritesParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("update_terrain", "return new UpdateTerrainParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("draw_debug", "return new DrawDebugParams(this.view.buffer, this.view.byteOffset + 4);"),
        ]
    );

    source
}

