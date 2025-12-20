use crate::data::assets::Texture;
use crate::data::gui::generate_sprites::GuiSpriteFlags;
use crate::data::gui::components::{GuiComponentView, GuiNode};
use crate::data::sprites::StaticSprite;
use crate::shared::{pos, rgba8, ExternalId, PositionF32, AABB};
use super::*;

fn no_texture() -> ExternalId { ExternalId(u32::MAX) }
fn texture(n: u32) -> Texture { Texture { id: ExternalId(n) } }
fn texture_id(n: u32) -> ExternalId { ExternalId(n) }
fn aabb(left: f32, top: f32, right: f32, bottom: f32) -> AABB { AABB { left, top, right, bottom } }

fn assert_sprite(sprite: &GuiOutputSprite, flags: GuiSpriteFlags, font_id: ExternalId, image_id: ExternalId, texcoord: AABB, positions: AABB) {
    assert_eq!(sprite.flags, flags);
    assert_eq!(sprite.font_texture_id, font_id);
    assert_eq!(sprite.image_texture_id, image_id);
    assert_eq!(sprite.texcoord, texcoord);
    assert_eq!(sprite.positions, positions);
}

fn assert_node(node: &GuiNode, children_count: u32, descendants_count: u32) {
    assert_eq!(node.children_count, children_count);
    assert_eq!(node.descendants_count, descendants_count);
}

fn assert_view(view: &GuiComponentView, positions: PositionF32, size: SizeF32, items_size: SizeF32) {
    assert_eq!(view.position, positions, "position mismatch");
    assert_eq!(view.size, size, "size mismatch");
    assert_eq!(view.items_size, items_size, "item size mismatch");
}

fn default_gui() -> Gui {
    let mut gui = Gui::default();
    gui.view_size = size(1000.0, 1000.0);
    return gui;
}

fn collect_sprites(gui: &mut Gui, cap: usize) -> Vec<GuiOutputSprite> {
    let mut collected = Vec::with_capacity(cap);
    gui.generate_sprites(|output| { collected.push(*output); });
    assert_eq!(collected.len(), cap);
    return collected;
}

#[test]
fn gui_basics() {
    let texcoord1 = aabb(10.0, 10.0, 110.0, 110.0);
    let texcoord2 = aabb(0.0, 0.0, 50.0, 50.0);
    let mut gui = default_gui();

    gui.build(|gui| {
        gui.layout_center();
        gui.image(texture(1), StaticSprite { texcoord: texcoord1 } );

        gui.layout_background();
        gui.image(texture(2), StaticSprite { texcoord: texcoord2 } );
    });

    assert_eq!(gui.components.len(), 2);

    let collected = collect_sprites(&mut gui, 2);
    
    assert_node(gui.components.get_node(0), 0, 0);
    assert_node(gui.components.get_node(1), 0, 0);

    assert_view(gui.components.get_view(0), pos(450.0, 450.0), size(100.0, 100.0), size(0.0, 0.0));
    assert_view(gui.components.get_view(1), pos(0.0, 0.0), size(1000.0, 1000.0), size(0.0, 0.0));

    assert_sprite(&collected[0], GuiSpriteFlags::TEXTURED, no_texture(), texture_id(1), texcoord1, aabb(450.0, 450.0, 550.0, 550.0));
    assert_sprite(&collected[1], GuiSpriteFlags::TEXTURED, no_texture(), texture_id(2), texcoord2, aabb(0.0, 0.0, 1000.0, 1000.0));
}

#[test]
fn gui_group() {
    let texcoord1 = aabb(10.0, 10.0, 110.0, 110.0);
    let mut gui = default_gui();

    gui.build(|gui| {
        gui.layout_center();
        gui.group(|gui| {
            gui.layout_center();
            gui.image(texture(1), StaticSprite { texcoord: texcoord1 } );
        });
    });

    assert_eq!(gui.components.len(), 2);

    let collected = collect_sprites(&mut gui, 1);

    assert_eq!(collected.len(), 1);
    assert_sprite(&collected[0], GuiSpriteFlags::TEXTURED, no_texture(), texture_id(1), texcoord1, aabb(450.0, 450.0, 550.0, 550.0));

    assert_node(gui.components.get_node(0), 1, 1);
    assert_view(gui.components.get_view(0), pos(450.0, 450.0), size(100.0, 100.0), size(100.0, 100.0));
    assert_view(gui.components.get_view(1), pos(450.0, 450.0), size(100.0, 100.0), size(0.0, 0.0));
}

#[test]
fn gui_flexbox_layout() {
    let texcoord = aabb(0.0, 0.0, 500.0, 200.0);
    let flex = FlexboxItemsLayout {
        children_offset: pos(0.0, 0.0),
        direction: FlexDirection::Column,
        align_items: FlexAlignItems::Center,
        justify_content: FlexJustifyContent::Start,
    };

    let mut gui = default_gui();

    gui.build(|gui| {
        gui.layout_center();
        gui.layout_items_flex(flex);
        gui.group(|gui| {
            gui.layout_background();
            gui.solid_color_block(rgba8(255, 255, 255, 255));

            gui.image(texture(1), StaticSprite { texcoord });

            gui.spacer(0.0, 100.0);

            gui.image(texture(1), StaticSprite { texcoord });
        });
    });

    assert_eq!(gui.components.len(), 5);

    let collected = collect_sprites(&mut gui, 3);
    assert_eq!(collected.len(), 3);
    assert_sprite(&collected[0], GuiSpriteFlags::SOLID_COLOR, no_texture(), no_texture(), AABB::default(), aabb(250.0, 250.0, 750.0, 750.0));
    assert_sprite(&collected[1], GuiSpriteFlags::TEXTURED, no_texture(), texture_id(1), texcoord, aabb(250.0, 250.0, 750.0, 450.0));
    assert_sprite(&collected[2], GuiSpriteFlags::TEXTURED, no_texture(), texture_id(1), texcoord, aabb(250.0, 550.0, 750.0, 750.0));
}

#[test]
fn test_flexbox_stretch() {
    let mut gui = default_gui();
    let flex = FlexboxItemsLayout {
        children_offset: pos(0.0, 0.0),
        direction: FlexDirection::Column,
        align_items: FlexAlignItems::Stretch,
        justify_content: FlexJustifyContent::Start,
    };

    gui.build(|gui| {
        gui.layout_parent_fixed_width(1000.0);
        gui.layout_items_flex(flex);
        gui.group(|gui| {
            gui.solid_color_block(rgba8(255, 255, 255, 255));
            gui.solid_color_block(rgba8(255, 255, 255, 255));
            gui.solid_color_block(rgba8(255, 255, 255, 255));
        });
    });

    assert_eq!(gui.components.len(), 4);

    let collected = collect_sprites(&mut gui, 3);
    assert_eq!(collected.len(), 3);

    assert_sprite(&collected[0], GuiSpriteFlags::SOLID_COLOR, no_texture(), no_texture(), AABB::default(), aabb(0.0, 0.0, 1000.0, 0.0));
    assert_sprite(&collected[1], GuiSpriteFlags::SOLID_COLOR, no_texture(), no_texture(), AABB::default(), aabb(0.0, 0.0, 1000.0, 0.0));
    assert_sprite(&collected[2], GuiSpriteFlags::SOLID_COLOR, no_texture(), no_texture(), AABB::default(), aabb(0.0, 0.0, 1000.0, 0.0));
}
