use hecs::{Entity, World as HecsWorld};
use zerocopy::transmute;
use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::shared::PositionF32;
use crate::store::StoreLoad;
use super::base::{BaseSprite, BaseSpriteFlags, AnimationState};

#[derive(Default)]
pub struct IsPawn;

/**
    Utility wrapper over `HecsWorld`. Think of it as this game's database.
*/
pub struct World {
    inner: HecsWorld
}

impl World {

    pub fn iter_all_sprites(&mut self) -> hecs::QueryMut<'_, &BaseSprite> {
        self.inner.query_mut::<&BaseSprite>()
    }

    pub fn iter_animated_sprites(&mut self) -> hecs::QueryMut<'_, (&mut BaseSprite, &mut AnimationState)> {
        self.inner.query_mut::<(&mut BaseSprite, &mut AnimationState)>()
    }

    pub fn iter_static_sprites(&mut self) -> hecs::QueryMut<'_, hecs::Without<&BaseSprite, &AnimationState>> {
        self.inner.query_mut().without::<&AnimationState>()
    }

    pub(super) fn add_pawn(&mut self, position: PositionF32, animate: AnimationState) -> Entity {
        let sprites = BaseSprite {
            position,
            texcoord: animate.current_frame(),
            flags: BaseSpriteFlags::empty(),
        };

        self.inner.spawn((IsPawn, sprites, animate))
    }

}

impl StoreLoad for World {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        let mut sprites = Vec::with_capacity(16);
        store_actors::<&IsPawn>(writer, &mut self.inner, &mut sprites);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut world = World::default();
        spawn_actors::<IsPawn>(reader, &mut world.inner);
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

fn store_actors<T: hecs::Query>(
    writer: &mut crate::store::StoreWriter,
    world: &mut HecsWorld,
    buffer: &mut Vec<EncodeActor>,
) {
    for (entity, (_, &sprite, &animate)) in world.query_mut::<(T, &BaseSprite, &AnimationState)>() {
        //dbg!("Storing {:?}", sprite);
        buffer.push(EncodeActor {
            entity: transmute!(entity.to_bits()),
            sprite,
            animate
        });
    }

    writer.write_array(&buffer);
    buffer.clear();
}

fn spawn_actors<T: hecs::Component + Default>(
    reader: &mut crate::store::StoreReader,
    world: &mut HecsWorld,
) {
    let actors = reader.read_array::<EncodeActor>();
    for actor in actors.iter() {
        //dbg!("Loading {:?}", actor.sprite);
        let entity = Entity::from_bits(transmute!(actor.entity)).expect("Corrupted entity data");
        world.spawn_at(entity, (T::default(), actor.sprite, actor.animate));
    }
}

//
// Other impl
//

impl Default for World {
    fn default() -> Self {
        World {
            inner: HecsWorld::default(),
        }
    }
}
