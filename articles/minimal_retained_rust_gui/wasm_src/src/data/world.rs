mod store;

use hecs::{Entity, World as HecsWorld};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};
use crate::shared::{PositionF32, AABB};
use super::sprites::{BaseSprite, AnimationState, StaticSprite};


#[derive(Default)] pub struct IsCastle;
#[derive(Default)] pub struct IsTower;
#[derive(Default)] pub struct IsHouse;
#[derive(Default)] pub struct IsKnight;

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

    pub(super) fn add_house(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        self.inner.spawn((IsHouse, HasCollision, BaseSprite::from_position_static(position, sprite)))
    }

    pub(super) fn add_castle(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        self.inner.spawn((IsCastle, HasCollision, BaseSprite::from_position_static(position, sprite)))
    }

    pub(super) fn add_tower(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        self.inner.spawn((IsTower, HasCollision, BaseSprite::from_position_static(position, sprite)))
    }

    pub(super) fn add_warrior(&mut self, position: PositionF32, animate: AnimationState) -> Entity {
        let sprite = BaseSprite::from_position_static(position, animate.current_frame());
        self.inner.spawn((IsKnight, sprite, animate))
    }

    pub fn sprite_at_position(&mut self, position: PositionF32) -> Option<Entity> {
        self.sprites_by_y_component.iter().rev()
            .find(|ordered_sprite| ordered_sprite.sprite.rect().point_inside(position) )
            .map(|sprite| sprite.e )
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
