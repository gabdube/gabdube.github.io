//! Store / Load logic for the World
use hecs::{Entity, World as HecsWorld};
use zerocopy::transmute;
use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::data::behaviour::{PawnBehaviourState, StorePawnBehaviour};
use super::{World, BaseSprite, AnimationState, IsPawn, IsCastle, IsHouse, HasCollision};

#[derive(Copy, Clone, IntoBytes, TryFromBytes, Immutable)]
pub struct StorePawn {
    entity: [u32; 2],
    sprite: BaseSprite,
    animate: AnimationState,
    behaviour: StorePawnBehaviour,
}

#[derive(Copy, Clone, IntoBytes, TryFromBytes, Immutable)]
pub struct StoreActor {
    entity: [u32; 2],
    sprite: BaseSprite,
}


impl crate::store::StoreLoad for World {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        store_pawns(writer, &mut self.inner);
        store_actors::<&IsHouse>(writer, &mut self.inner);
        store_actors::<&IsCastle>(writer, &mut self.inner);
        writer.write_option(&self.insert_sprite);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut world = World::default();
        spawn_pawns(reader, &mut world.inner);
        spawn_actors::<IsHouse>(reader, &mut world.inner);
        spawn_actors::<IsCastle>(reader, &mut world.inner);
        world.insert_sprite = reader.try_read_option()?;

        Ok(world)
    }
}


fn store_pawns(writer: &mut crate::store::StoreWriter, world: &mut HecsWorld) {
    let mut sprites = Vec::with_capacity(16);
    
    for (entity, (_, &sprite, &animate, &behaviour)) in world.query_mut::<(&IsPawn, &BaseSprite, &AnimationState, &PawnBehaviourState)>() {
        sprites.push(StorePawn {
            entity: transmute!(entity.to_bits()),
            sprite,
            animate,
            behaviour: StorePawnBehaviour::from(behaviour),
        });
    }

    writer.write_array(&sprites);
    sprites.clear();
}

fn store_actors<T: hecs::Query>(writer: &mut crate::store::StoreWriter, world: &mut HecsWorld) {
    let mut sprites = Vec::with_capacity(16);

    for (entity, (_, &sprite)) in world.query_mut::<(T, &BaseSprite)>() {
        sprites.push(StoreActor {
            entity: transmute!(entity.to_bits()),
            sprite,
        });
    }

    writer.write_array(&sprites);
    sprites.clear();
}

fn spawn_pawns(
    reader: &mut crate::store::StoreReader,
    world: &mut HecsWorld,
) {
    let actors = unsafe { reader.read_array_transmute::<StorePawn>() };
    world.reserve::<(IsPawn, BaseSprite, AnimationState, PawnBehaviourState)>(actors.len() as u32);
    for actor in actors.iter() {
        let entity = Entity::from_bits(transmute!(actor.entity)).expect("Corrupted entity data");
        let behaviour = PawnBehaviourState::from(actor.behaviour);
        world.spawn_at(entity, (IsPawn, actor.sprite, actor.animate, behaviour));
    }
}

fn spawn_actors<T: hecs::Component + Default>(
    reader: &mut crate::store::StoreReader,
    world: &mut HecsWorld,
) {
    let actors = unsafe { reader.read_array_transmute::<StoreActor>() };
    world.reserve::<(T, BaseSprite)>(actors.len() as u32);
    for actor in actors.iter() {
        let entity = Entity::from_bits(transmute!(actor.entity)).expect("Corrupted entity data");
        world.spawn_at(entity, (T::default(), HasCollision, actor.sprite));
    }
}
