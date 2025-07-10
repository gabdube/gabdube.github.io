use hecs::Entity;

use crate::shared::{PositionF32, AABB};
use crate::data::sprites::BaseSprite;
use super::World;

#[derive(Default, Copy, Clone)]
pub struct SelectionFlags(pub u32);

impl SelectionFlags {
    pub const HAS_FRIENDLY_UNITS: u32 = 0x1;

    flags!(has_friendly_units, set_has_friendly_units, Self::HAS_FRIENDLY_UNITS);
}

impl World {
    pub fn selected_count(&self) -> usize {
        self.selected_sprites.len()
    }

    pub fn selected_sprites(&self) -> &[Entity] {
        &self.selected_sprites
    }

    pub fn select_sprite_at_position(&mut self, position: PositionF32) {
        if let Some(entity) = self.sprite_at_position(position) {
            if let Ok(sprite) = self.inner.query_one_mut::<&mut BaseSprite>(entity) {
                sprite.flags.set_highlighted();
                sprite.highlight_color = [255; 3];
                self.selected_sprites.push(entity);
            }
        }
    }

    pub fn select_sprites_rect(&mut self, selection: &AABB) {
        for (entity, sprite) in self.inner.query_mut::<&mut BaseSprite>() {
            if sprite.rect().intersects(selection) {
                sprite.flags.set_highlighted();
                sprite.highlight_color = [255; 3];
                self.selected_sprites.push(entity);
            }
        }
    }

    pub fn clear_selected_sprites(&mut self) {
        if self.selected_sprites.is_empty() {
            return;
        }

        for &entity in self.selected_sprites.iter() {
            if let Ok(sprite) = self.inner.query_one_mut::<&mut BaseSprite>(entity) {
                sprite.flags.clear_highlighted();
                sprite.highlight_color = [0; 3];
            }
        }

        self.selected_sprites.clear();
    }


    pub fn compute_selection_flags(&mut self) -> SelectionFlags {
        let mut flags = SelectionFlags::default();


        flags
    }
}
