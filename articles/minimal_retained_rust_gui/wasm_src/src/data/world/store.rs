//! Store / Load logic for the World
use hecs::{Entity, World as HecsWorld};
use zerocopy::transmute;
use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::{data::behaviour::KnightBehaviourState, store::StoreLoad};
use super::{World, BaseSprite, HasCollision, AnimationState, tags::*};

#[derive(Copy, Clone, IntoBytes, TryFromBytes, Immutable)]
pub struct StoreActor {
    entity: [u32; 2],
    sprite: BaseSprite,
}

impl crate::store::StoreLoad for World {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        store_knights(writer, &mut self.inner);
        store_actors::<&IsHouse>(writer, &mut self.inner);
        store_actors::<&IsCastle>(writer, &mut self.inner);
        store_actors::<&IsTower>(writer, &mut self.inner);
        store_selected(writer, self);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut world = World::default();
        spawn_knights(reader, &mut world.inner)?;
        spawn_actors::<IsHouse>(reader, &mut world.inner, EntityId::HOUSE);
        spawn_actors::<IsCastle>(reader, &mut world.inner, EntityId::CASTLE);
        spawn_actors::<IsTower>(reader, &mut world.inner, EntityId::TOWER);
        load_selected(reader, &mut world)?;

        Ok(world)
    }
}

fn store_knights(writer: &mut crate::store::StoreWriter, world: &mut HecsWorld) {
    let count = world.query_mut::<(&IsKnight, &BaseSprite, &AnimationState)>().into_iter().count() as u32;
    writer.write(&count);

    for (entity, (_, sprite, animate, behaviour)) in world.query_mut::<(&IsKnight, &BaseSprite, &AnimationState, &mut KnightBehaviourState)>() {
        writer.write_entity_option(Some(entity));
        writer.write(sprite);
        writer.write(animate);
        behaviour.store(writer);
    }
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

fn store_selected(writer: &mut crate::store::StoreWriter, world: &mut World) {
    let selected_count = world.selected_sprites.len() as u32;
    writer.write(&selected_count);

    for &entity in world.selected_sprites.iter() {
        writer.write_entity_option(Some(entity));
    }
}

fn spawn_knights(reader: &mut crate::store::StoreReader, world: &mut HecsWorld) -> Result<(), crate::error::Error> {
    let count: u32 = reader.try_read()?;
    world.reserve::<(IsKnight, BaseSprite, AnimationState)>(count);
    for _ in 0..count {
        let entity = reader.try_read_entity_option()?.expect("Corrupted entity");
        let sprite: BaseSprite = reader.try_read()?;
        let animate: AnimationState = reader.try_read()?;
        let behaviour = KnightBehaviourState::load(reader)?;
        let id = EntityId::KNIGHT;
        world.spawn_at(entity, (IsKnight, id, sprite, animate, behaviour));
    }

    Ok(())
}

fn spawn_actors<T: hecs::Component + Default>(
    reader: &mut crate::store::StoreReader,
    world: &mut HecsWorld,
    id: EntityId,
) {
    let actors = unsafe { reader.read_array_transmute::<StoreActor>() };
    world.reserve::<(T, BaseSprite)>(actors.len() as u32);
    for actor in actors.iter() {
        let entity = Entity::from_bits(transmute!(actor.entity)).expect("Corrupted entity data");
        world.spawn_at(entity, (T::default(), HasCollision, id, actor.sprite));
    }
}

fn load_selected(reader: &mut crate::store::StoreReader, world: &mut World) -> Result<(), crate::error::Error> {
    let selected_count: u32 = reader.try_read()?;
    world.selected_sprites = Vec::with_capacity(selected_count as usize);

    for _ in 0..selected_count {
        if let Ok(Some(entity)) = reader.try_read_entity_option() {
            world.selected_sprites.push(entity);
        }
    }

    Ok(())
}
