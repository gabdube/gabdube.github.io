mod store;
mod behaviour;

use hecs::{Entity, World as HecsWorld};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};
use crate::shared::{PositionF32, AABB};
use super::base::{BaseSprite, BaseSpriteFlags, AnimationState, StaticSprite};
use super::behaviour::PawnBehaviourState;

#[derive(Default)] pub struct IsPawn;
#[derive(Default)] pub struct IsCastle;
#[derive(Default)] pub struct IsHouse;

#[derive(Default)] pub struct HasCollision;

#[derive(Copy, Clone, IntoBytes, FromBytes, Immutable)]
pub struct InsertSprite {
    pub position: PositionF32,
    pub sprite: AABB,
}

#[derive(Copy, Clone)]
pub struct OrderedSprite {
    pub e: Entity,
    pub y: f32,
    pub sprite: BaseSprite,
}

/**
    Utility wrapper over `HecsWorld`. This is basically the game database.
*/
pub struct World {
    inner: HecsWorld,
    // The sprite displayed when currently inserting new elements in the game
    insert_sprite: Option<InsertSprite>,
    // Quick lookup for the selected sprites
    selected_sprites: Vec<Entity>,
    // Sprites ordered by Y component. For rendering purpose
    sprites_by_y_component: Vec<OrderedSprite>,
}

impl World {

    pub fn selected_sprites(&self) -> &Vec<Entity> {
        &self.selected_sprites
    }

    pub fn has_insert_sprite(&self) -> Option<InsertSprite> {
        self.insert_sprite
    }

    /// Renders a half transparent static sprite at `position`
    pub fn set_insert_sprite(&mut self, position: PositionF32, sprite: StaticSprite) {
        self.insert_sprite = Some(InsertSprite { position, sprite: sprite.texcoord });
    }

    pub fn clear_insert_sprite(&mut self) {
        self.insert_sprite = None;
    }

    pub fn sprite_at_position(&mut self, position: PositionF32) -> Option<Entity> {
        self.sprites_by_y_component.iter().rev()
            .find(|ordered_sprite| ordered_sprite.sprite.rect().point_inside(position) )
            .map(|sprite| sprite.e )
    }

    pub fn delete_sprite_at_position(&mut self, position: PositionF32) -> bool {
        if let Some(e1) = self.sprite_at_position(position) {
            if let Some(index) = self.selected_sprites.iter().position(|&e2| e2 == e1 ) {
                self.selected_sprites.remove(index);
            }

            if let Err(err) = self.inner.despawn(e1) {
                dbg!("Failed to remove entity {:?}", err);
                return false;
            }

            true
        } else {
            false
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

    pub fn select_sprite_at_position(&mut self, position: PositionF32) {
        if let Some(entity) = self.sprite_at_position(position) {
            if let Ok(sprite) = self.inner.query_one_mut::<&mut BaseSprite>(entity) {
                sprite.flags.set_highlighted();
                sprite.highlight_color = [255; 3];
                self.selected_sprites.push(entity);
                // dbg!("Selected {:?}", entity);
            }
        }
    }

    pub fn clear_sprite_highlight(&mut self, entity: Entity) {
        if let Ok(mut sprite) = self.inner.get::<&mut BaseSprite>(entity) {
            sprite.flags.clear_highlighted();
            sprite.highlight_color = [0; 3];
        }
    }

    pub fn set_sprite_highlight(&mut self, entity: Entity, color: [u8; 3]) {
        if let Ok(mut sprite) = self.inner.get::<&mut BaseSprite>(entity) {
            sprite.flags.set_highlighted();
            sprite.highlight_color = color;
        }
    }

    pub fn sprites_with_collisions(&self) -> hecs::QueryBorrow<(&BaseSprite, &HasCollision)> {
        self.inner.query::<(&BaseSprite, &HasCollision)>()
    }

    pub(super) fn add_pawn(&mut self, position: PositionF32, animate: AnimationState) -> Entity {
        let sprites = BaseSprite {
            position,
            texcoord: animate.current_frame(),
            highlight_color: [0, 0, 0],
            flags: BaseSpriteFlags::empty(),
        };

        self.inner.spawn((IsPawn, sprites, animate, PawnBehaviourState::idle()))
    }

    pub fn is_pawn(&self, entity: Entity) -> bool {
        match self.inner.query_one::<&IsPawn>(entity).ok() {
            Some(mut v) => v.get().is_some(),
            None => false
        }
    }

    pub fn get_pawn_position(&self, entity: Entity) -> Option<PositionF32> {
        let mut sprite_query = self.inner.query_one::<(&IsPawn, &mut BaseSprite)>(entity).ok()?;
        let (_, sprite) = sprite_query.get()?;
        Some(sprite.base_position())
    }

    pub fn set_pawn_position(&self, entity: Entity, position: PositionF32) -> Option<()> {
        let mut query = self.inner.query_one::<(&IsPawn, &mut BaseSprite)>(entity).ok()?;
        let (_, sprite) = query.get()?;
        sprite.set_base_position(position);
        Some(())
    }

    pub fn set_pawn_flipped(&self, entity: Entity, flipped: bool) -> Option<()> {
        let mut query = self.inner.query_one::<(&IsPawn, &mut BaseSprite)>(entity).ok()?;
        let (_, sprite) = query.get()?;
        match flipped {
            true => { sprite.flags.set_flipped(); }
            false => { sprite.flags.clear_flipped(); }
        }
        Some(())
    }

    pub fn set_pawn_animation(&self, entity: Entity, new_animation: AnimationState) -> Option<()> {
        let mut query = self.inner.query_one::<(&IsPawn, &mut AnimationState)>(entity).ok()?;
        let (_, animation) = query.get()?;
        *animation = new_animation;
        Some(())
    }

    pub(super) fn add_house(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        let sprites = BaseSprite {
            position,
            texcoord: sprite.texcoord,
            highlight_color: [0, 0, 0],
            flags: BaseSpriteFlags::empty(),
        };

        self.inner.spawn((IsHouse, HasCollision, sprites))
    }

    pub(super) fn add_castle(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        let sprites = BaseSprite {
            position,
            texcoord: sprite.texcoord,
            highlight_color: [0, 0, 0],
            flags: BaseSpriteFlags::empty(),
        };

        self.inner.spawn((IsCastle, HasCollision, sprites))
    }

    /// Order all sprites in the world by their y component
    /// Optionally advance the animation if `animate` is true
    pub fn order_sprites(&mut self, animate: bool) -> usize {
        use std::cmp::Ordering;

        fn copy_sprites(world: &mut World) {
            for (e, &sprite) in world.inner.query_mut::<&BaseSprite>() {
                world.sprites_by_y_component.push(OrderedSprite { e, y: sprite.position.y + sprite.texcoord.height(), sprite })
            }
        }

        fn copy_sprites_with_animations(world: &mut World) {
            for (e, (sprite, animation)) in world.inner.query_mut::<(&mut BaseSprite, &mut AnimationState)>() {
                animation.current_frame += 1;
                animation.current_frame = animation.current_frame * ((animation.current_frame < animation.max_frame) as u16);
                sprite.texcoord = animation.current_frame();
                world.sprites_by_y_component.push(OrderedSprite { e, y: sprite.position.y + sprite.texcoord.height(), sprite: *sprite })
            }
            
            for (e, &sprite) in world.inner.query_mut::<&BaseSprite>().without::<&AnimationState>() {
                world.sprites_by_y_component.push(OrderedSprite { e, y: sprite.position.y + sprite.texcoord.height(), sprite })
            }
        }

        self.sprites_by_y_component.clear();

        if animate {
            copy_sprites_with_animations(self);
        } else {
            copy_sprites(self);
        }

        self.sprites_by_y_component.sort_unstable_by(|v1, v2| {
            v1.y.partial_cmp(&v2.y).unwrap_or(Ordering::Equal)
        });

        self.sprites_by_y_component.len()
    } 

    pub fn ordered_sprites<'a>(&'a mut self) -> impl Iterator<Item=BaseSprite> + 'a {
        self.sprites_by_y_component.iter()
            .map(|ordered_sprite| ordered_sprite.sprite )
    }

}



//
// Other impl
//

impl Default for World {
    fn default() -> Self {
        World {
            inner: HecsWorld::default(),
            insert_sprite: None,
            selected_sprites: Vec::with_capacity(8),
            sprites_by_y_component: Vec::with_capacity(32),
        }
    }
}
