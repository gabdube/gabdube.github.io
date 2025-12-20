use crate::data::assets::TextMetrics;
use crate::shared::{ColorRGBA8, ExternalId, AABB, Scissor, aabb, pos, rgba8, size};
use super::components::*;
use super::Gui;

pub(crate) const NO_TEXTURE: ExternalId = ExternalId(u32::MAX);

#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct GuiSpriteFlags(pub u32);

impl GuiSpriteFlags {
    pub const TEXTURED: Self = Self(0);
    pub const SOLID_COLOR: Self = Self(1);
    pub const TEXT: Self = Self(2);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GuiOutputSprite {
    pub positions: AABB,
    pub texcoord: AABB,
    pub color: ColorRGBA8,
    pub flags: GuiSpriteFlags,
    pub image_texture_id: ExternalId,
    pub font_texture_id: ExternalId,
    pub scissor: Scissor,
}

impl Default for GuiOutputSprite {
    fn default() -> Self {
        GuiOutputSprite { 
            positions: AABB::default(),
            texcoord: AABB::default(),
            color: rgba8(255, 255, 0, 255),
            flags: GuiSpriteFlags::SOLID_COLOR,
            image_texture_id: ExternalId::default(),
            font_texture_id: ExternalId::default(),
            scissor: Scissor::default(),
        }
    }
}


pub(super) fn generate_sprites<F: FnMut(&GuiOutputSprite)>(gui: &mut Gui, mut callback: F) {
    let components = &gui.components;
    let component_count = components.len();

    for i in 0..component_count {
        let view = components.get_view(i);
        if view.clipped() {
            continue;
        }

        let data = components.get_data_mut(i);
        match data {
            GuiComponentData::Group |
            GuiComponentData::ListViewBase |
            GuiComponentData::Spacer(_) |
            GuiComponentData::ScrollView(_) |
            GuiComponentData::Window(_) => {},
            GuiComponentData::SolidColorBlock(block) => { block.generate_sprites(view, &mut callback) }
            GuiComponentData::Borders(borders) => { borders.generate_sprites(view, &mut callback); }
            GuiComponentData::Image(image) => { image.generate_sprites(view, &mut callback); }
            GuiComponentData::Label(label) => { label.generate_sprites(view, &mut callback); }
            GuiComponentData::Button(button) => { button.generate_sprites(view, &mut callback); }
            GuiComponentData::TextInput(text_input) => { text_input.generate_sprites(view, &mut callback); }
            GuiComponentData::ListViewItem(item) => { item.generate_sprites(view, &mut callback); }
            GuiComponentData::ScrollbarVertical(bar) => { bar.generate_sprites(view, &mut callback); }
            GuiComponentData::WindowTitleBar(bar) => { bar.generate_sprites(view, &mut callback) },
        }
    }
}

pub(crate) fn generate_solid_color_block<F: FnMut(&GuiOutputSprite)>(
    color: ColorRGBA8,
    view: &GuiComponentView,
    callback: &mut F
) {
    let positions = aabb(view.position, view.size);
    let sprite = GuiOutputSprite {
        positions,
        texcoord: AABB::default(),
        color,
        flags: GuiSpriteFlags::SOLID_COLOR,
        image_texture_id: NO_TEXTURE,
        font_texture_id: NO_TEXTURE,
        scissor: view.scissor,
    };

    callback(&sprite);
}

pub(crate) fn generate_image<F: FnMut(&GuiOutputSprite)>(
    texcoord: AABB,
    texture_id: ExternalId,
    view: &GuiComponentView,
    callback: &mut F
) {
    let positions = aabb(view.position, view.size);
    let sprite = GuiOutputSprite {
        positions,
        texcoord,
        color: rgba8(255, 255, 255, 255),
        flags: GuiSpriteFlags::TEXTURED,
        image_texture_id: texture_id,
        font_texture_id: NO_TEXTURE,
        scissor: view.scissor,
    };

    callback(&sprite);
}

pub(crate) fn generate_borders<F: FnMut(&GuiOutputSprite)>(view: &GuiComponentView, border_width: f32, color: ColorRGBA8, callback: &mut F) {
    let base = GuiOutputSprite { 
        positions: AABB::default(),
        texcoord: AABB::default(),
        color,
        flags: GuiSpriteFlags::SOLID_COLOR,
        font_texture_id: NO_TEXTURE,
        image_texture_id: NO_TEXTURE,
        scissor: view.scissor
    };

    let pos1 = view.position;
    let pos2 = pos(pos1.x, pos1.y + view.size.height - border_width);
    let pos3 = pos(pos1.x + view.size.width - border_width, pos1.y);

    let size1 = size(view.size.width, border_width);
    let size2 = size(border_width, view.size.height);

    callback(&GuiOutputSprite { positions: aabb(pos1, size1), ..base });
    callback(&GuiOutputSprite { positions: aabb(pos2, size1), ..base });
    callback(&GuiOutputSprite { positions: aabb(pos1, size2), ..base });
    callback(&GuiOutputSprite { positions: aabb(pos3, size2), ..base });
}

pub(crate) fn generate_text<F: FnMut(&GuiOutputSprite)>(text: &TextMetrics, view: &GuiComponentView, color: ColorRGBA8, callback: &mut F) {
    let [x, y] = view.position.splat();
    let scissor = view.scissor;

    let scissor_left = scissor.x as f32;
    let scissor_right = (scissor.x + scissor.width) as f32;

    for glyph in text.glyphs.iter() {
        let mut glyph_position = glyph.position;
        glyph_position.left += x;
        glyph_position.right += x;
        glyph_position.top += y;
        glyph_position.bottom += y;

        // Do not emit a glyph if it is outside the scissor
        if glyph_position.right < scissor_left || glyph_position.left > scissor_right {
            continue;
        }

        callback(&GuiOutputSprite { 
            positions: glyph_position,
            texcoord: glyph.texcoord,
            color,
            flags: GuiSpriteFlags::TEXT,
            image_texture_id: NO_TEXTURE,
            font_texture_id: text.texture.id,
            scissor,
        });
    }
}

