mod tags;
pub use tags::*;

mod select;
mod store;

use hecs::{World as HecsWorld, Entity, QueryBorrow};
use crate::data::behaviour::KnightBehaviourState;
use crate::shared::PositionF32;
use super::sprites::{BaseSprite, AnimationState, StaticSprite, OrderedSprite};

/**
    Utility wrapper over `HecsWorld`. This is basically the game database.
*/
pub struct World {
    inner: HecsWorld,
    // Quick lookup for the selected sprites
    selected_sprites: Vec<Entity>,
    // Sprites ordered by Y component. For rendering purpose
    sprites_by_y_component: Vec<OrderedSprite>,
}

impl World {

    pub(super) fn add_house(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        self.inner.spawn((IsHouse, HasCollision, EntityId::HOUSE, BaseSprite::from_position_static(position, sprite)))
    }

    pub(super) fn add_castle(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        self.inner.spawn((IsCastle, HasCollision, EntityId::CASTLE, BaseSprite::from_position_static(position, sprite)))
    }

    pub(super) fn add_tower(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        self.inner.spawn((IsTower, HasCollision, EntityId::TOWER, BaseSprite::from_position_static(position, sprite)))
    }

    pub(super) fn add_knight(&mut self, position: PositionF32) -> Entity {
        self.inner.spawn((IsKnight, EntityId::KNIGHT, BaseSprite::from_position(position), AnimationState::default(), KnightBehaviourState::idle()))
    }

    // Panics if `entity` is not a knight
    pub fn knight_behaviour_mut<'a>(&'a mut self, entity: Entity) -> &'a mut KnightBehaviourState {
        self.inner.query_one_mut::<&mut KnightBehaviourState>(entity).unwrap()
    }

    pub fn all_knights_behaviour<'a>(&self) -> QueryBorrow<&mut KnightBehaviourState> {
        self.inner.query::<&mut KnightBehaviourState>()
    }

    pub fn set_sprite_animation(&self, entity: Entity, new_animation: AnimationState) {
        if let Ok(mut query) = self.inner.query_one::<(&mut BaseSprite, &mut AnimationState)>(entity) {
            if let Some((sprite, animation)) = query.get() {
                sprite.texcoord = new_animation.current_frame().texcoord;
                *animation = new_animation;
            }
        }
    }

    pub fn sprite_base_position(&self, entity: Entity) -> Option<PositionF32> {
        let mut sprite_query = self.inner.query_one::<&BaseSprite>(entity).ok()?;
        let sprite = sprite_query.get()?;
        Some(sprite.base_position())
    }

    pub fn set_sprite_position(&self, entity: Entity, position: PositionF32, pixel_perfect: bool) {
        if let Ok(mut query) = self.inner.query_one::<&mut BaseSprite>(entity) {
            if let Some(sprite) = query.get() {
                 sprite.set_base_position(position, pixel_perfect);
            }
        }
    }

    pub fn flip_sprite(&self, entity: Entity, flipped: bool) {
        if let Ok(mut query) = self.inner.query_one::<&mut BaseSprite>(entity) {
            if let Some(sprite) = query.get() {
                match flipped {
                    true => { sprite.flags.set_flipped(); }
                    false => { sprite.flags.clear_flipped(); }
                }
            }
        }
    }

    pub fn sprite_at_position(&mut self, position: PositionF32) -> Option<Entity> {
        self.sprites_by_y_component.iter().rev()
            .find(|ordered_sprite| ordered_sprite.sprite.rect().point_inside(position) )
            .map(|sprite| sprite.e )
    }

    pub fn sprites_with_collisions(&self) -> hecs::QueryBorrow<(&BaseSprite, &HasCollision)> {
        self.inner.query::<(&BaseSprite, &HasCollision)>()
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
                sprite.texcoord = animation.current_frame().texcoord;
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

    pub fn entity_id_mut(&mut self, entity: Entity) -> EntityId {
        self.inner.query_one_mut::<&EntityId>(entity)
            .map(|id| *id )
            .unwrap()
    }

}


//
// Other impl
//

impl Default for World {
    fn default() -> Self {
        World {
            inner: HecsWorld::default(),
            selected_sprites: Vec::with_capacity(8),
            sprites_by_y_component: Vec::with_capacity(32),
        }
    }
}
