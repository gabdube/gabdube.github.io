use hecs::{Entity, World as HecsWorld};
use zerocopy::transmute;
use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::shared::{PositionF32, AABB};
use crate::store::StoreLoad;
use super::base::{BaseSprite, BaseSpriteFlags, AnimationState, StaticSprite};

#[derive(Default)] pub struct IsPawn;
#[derive(Default)] pub struct IsCastle;
#[derive(Default)] pub struct IsHouse;

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
    Utility wrapper over `HecsWorld`. Think of it as this game's database.
*/
pub struct World {
    inner: HecsWorld,
    insert_sprite: Option<InsertSprite>,
    selected_sprites: Vec<Entity>,
    sprites_by_y_component: Vec<OrderedSprite>,
}

impl World {

    /// Renders a half transparent static sprite at `position`
    pub fn set_insert_sprite(&mut self, position: PositionF32, sprite: StaticSprite) {
        self.insert_sprite = Some(InsertSprite { position, sprite: sprite.texcoord });
    }

    pub fn has_insert_sprite(&self) -> Option<InsertSprite> {
        self.insert_sprite
    }

    pub fn clear_insert_sprite(&mut self) {
        self.insert_sprite = None;
    }

    pub fn sprite_at_position(&mut self, position: PositionF32) -> Option<Entity> {
        self.sprites_by_y_component.iter().rev()
            .find(|ordered_sprite| ordered_sprite.sprite.rect().point_inside(position) )
            .map(|sprite| sprite.e )
    }

    pub fn delete_sprite_at_position(&mut self, position: PositionF32) {
        if let Some(e1) = self.sprite_at_position(position) {
            if let Some(index) = self.selected_sprites.iter().position(|&e2| e2 == e1 ) {
                self.selected_sprites.remove(index);
            }

            if let Err(err) = self.inner.despawn(e1) {
                dbg!("Failed to remove entity {:?}", err);
            }
        }
    }

    pub fn clear_selected_sprites(&mut self) {
        if self.selected_sprites.is_empty() {
            return;
        }

        for &entity in self.selected_sprites.iter() {
            if let Ok(mut sprite) = self.inner.get::<&mut BaseSprite>(entity) {
                sprite.flags.clear_highlighted();
                sprite.highlight_color = [0; 3];
            }
        }

        self.selected_sprites.clear();
    }

    pub fn select_sprite_at_position(&mut self, position: PositionF32) {
        if let Some(entity) = self.sprite_at_position(position) {
            if let Ok(mut sprite) = self.inner.get::<&mut BaseSprite>(entity) {
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

    pub(super) fn add_pawn(&mut self, position: PositionF32, animate: AnimationState) -> Entity {
        let sprites = BaseSprite {
            position,
            texcoord: animate.current_frame(),
            highlight_color: [0, 0, 0],
            flags: BaseSpriteFlags::empty(),
        };

        self.inner.spawn((IsPawn, sprites, animate))
    }

    pub(super) fn add_house(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        let sprites = BaseSprite {
            position,
            texcoord: sprite.texcoord,
            highlight_color: [0, 0, 0],
            flags: BaseSpriteFlags::empty(),
        };

        self.inner.spawn((IsHouse, sprites))
    }

    pub(super) fn add_castle(&mut self, position: PositionF32, sprite: StaticSprite) -> Entity {
        let sprites = BaseSprite {
            position,
            texcoord: sprite.texcoord,
            highlight_color: [0, 0, 0],
            flags: BaseSpriteFlags::empty(),
        };

        self.inner.spawn((IsCastle, sprites))
    }

    /// Order all sprites in the world by their y components
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

impl StoreLoad for World {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        let mut sprites = Vec::with_capacity(16);
        store_actors_animated::<&IsPawn>(writer, &mut self.inner, &mut sprites);
        store_actors::<&IsHouse>(writer, &mut self.inner, &mut sprites);
        store_actors::<&IsCastle>(writer, &mut self.inner, &mut sprites);
        writer.write_option(&self.insert_sprite);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut world = World::default();
        spawn_actors_animated::<IsPawn>(reader, &mut world.inner);
        spawn_actors::<IsHouse>(reader, &mut world.inner);
        spawn_actors::<IsCastle>(reader, &mut world.inner);
        world.insert_sprite = reader.try_read_option()?;
        Ok(world)
    }
}

//
// Store / Load
//

#[derive(Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct EncodeActor {
    entity: [u32; 2],
    sprite: BaseSprite,
    animate: AnimationState,
}

fn store_actors_animated<T: hecs::Query>(
    writer: &mut crate::store::StoreWriter,
    world: &mut HecsWorld,
    sprites: &mut Vec<EncodeActor>,
) {
    
    for (entity, (_, &sprite, &animate)) in world.query_mut::<(T, &BaseSprite, &AnimationState)>() {
        sprites.push(EncodeActor {
            entity: transmute!(entity.to_bits()),
            sprite,
            animate
        });
    }

    writer.write_array(&sprites);
    sprites.clear();
}

fn store_actors<T: hecs::Query>(
    writer: &mut crate::store::StoreWriter,
    world: &mut HecsWorld,
    sprites: &mut Vec<EncodeActor>,
) {
    
    for (entity, (_, &sprite)) in world.query_mut::<(T, &BaseSprite)>() {
        sprites.push(EncodeActor {
            entity: transmute!(entity.to_bits()),
            sprite,
            animate: Default::default(),
        });
    }

    writer.write_array(&sprites);
    sprites.clear();
}

fn spawn_actors_animated<T: hecs::Component + Default>(
    reader: &mut crate::store::StoreReader,
    world: &mut HecsWorld,
) {
    let actors = reader.read_array::<EncodeActor>();
    world.reserve::<(T, BaseSprite, AnimationState)>(actors.len() as u32);
    for actor in actors.iter() {
        let entity = Entity::from_bits(transmute!(actor.entity)).expect("Corrupted entity data");
        world.spawn_at(entity, (T::default(), actor.sprite, actor.animate));
    }
}

fn spawn_actors<T: hecs::Component + Default>(
    reader: &mut crate::store::StoreReader,
    world: &mut HecsWorld,
) {
    let actors = reader.read_array::<EncodeActor>();
    world.reserve::<(T, BaseSprite)>(actors.len() as u32);
    for actor in actors.iter() {
        let entity = Entity::from_bits(transmute!(actor.entity)).expect("Corrupted entity data");
        world.spawn_at(entity, (T::default(), actor.sprite));
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
