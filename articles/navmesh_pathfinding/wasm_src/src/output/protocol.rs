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

fn get_array_function(accessor: &str, base_offset: usize, item_size: usize, item_count: usize) -> String {
    let mut out = String::with_capacity(32);
    out.push_str("return [");
    for i in 0..item_count {
        let offset = base_offset + ((i * item_size) as usize);
        out.push_str(&format!("this.view.{accessor}({offset}, true),"))
    }
    out.push_str("];");
    out
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
            ("UpdateHighlightSprites", OutputMessageType::UpdateHighlightSprites),
            ("HighlightSprites", OutputMessageType::HighlightSprites),
            ("UpdateTerrain", OutputMessageType::UpdateTerrain),
            ("DrawDebug", OutputMessageType::DrawDebug),
            ("GuiTextureUpdate", OutputMessageType::GuiTextureUpdate),
            ("GuiMeshUpdate", OutputMessageType::GuiMeshUpdate),
            ("ResetGui", OutputMessageType::ResetGui),
            ("UpdateViewOffset", OutputMessageType::UpdateViewOffset),
            ("DrawInsertSprite", OutputMessageType::DrawInsertSprite),
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

    generate_struct(
        &mut source, 
        "GuiTextureUpdateParams", 
        size_of::<GuiTextureUpdateParams>(),
        &[
            ("pixels_offset", pointer_type, offset_of!(GuiTextureUpdateParams, pixels_offset)),
            ("pixels_size", pointer_type, offset_of!(GuiTextureUpdateParams, pixels_size)),
            ("x", "getUint32", offset_of!(GuiTextureUpdateParams, x)),
            ("y", "getUint32", offset_of!(GuiTextureUpdateParams, y)),
            ("width", "getUint32", offset_of!(GuiTextureUpdateParams, width)),
            ("height", "getUint32", offset_of!(GuiTextureUpdateParams, height)),
            ("id", "getUint32", offset_of!(GuiTextureUpdateParams, id)),
        ],
    );

    generate_struct_with_custom_fields(
        &mut source, 
        "GuiMeshUpdateParams", 
        size_of::<GuiMeshUpdateParams>(),
        &[
            ("index_offset_bytes", pointer_type, offset_of!(GuiMeshUpdateParams, index_offset_bytes)),
            ("index_size_bytes", pointer_type, offset_of!(GuiMeshUpdateParams, index_size_bytes)),
            ("vertex_offset_bytes", pointer_type, offset_of!(GuiMeshUpdateParams, vertex_offset_bytes)),
            ("vertex_size_bytes", pointer_type, offset_of!(GuiMeshUpdateParams, vertex_size_bytes)),
            ("count", "getUint32", offset_of!(GuiMeshUpdateParams, count)),
            ("texture_id", "getUint32", offset_of!(GuiMeshUpdateParams, texture_id)),
        ],
        &[
            ("clip", &get_array_function("getFloat32", offset_of!(GuiMeshUpdateParams, clip), size_of::<f32>(), 4))
        ]
    );

    generate_struct(
        &mut source, 
        "DrawInsertSpriteParams", 
        size_of::<DrawInsertSpriteParams>(),
        &[
            ("vertex_offset_bytes", pointer_type, offset_of!(DrawInsertSpriteParams, vertex_offset_bytes)),
            ("vertex_size_bytes", pointer_type, offset_of!(DrawInsertSpriteParams, vertex_size_bytes)),
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
            ("update_highlight_sprites", "return new UpdateSpritesParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("draw_highlight_sprites", "return new DrawSpritesParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("draw_insert_sprite", "return new DrawInsertSpriteParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("update_terrain", "return new UpdateTerrainParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("draw_debug", "return new DrawDebugParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("gui_texture_update", "return new GuiTextureUpdateParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("gui_mesh_update", "return new GuiMeshUpdateParams(this.view.buffer, this.view.byteOffset + 4);"),
            ("update_view_offset", "return [this.view.getFloat32(4, true), this.view.getFloat32(8, true)];"),
        ]
    );

    source
}

