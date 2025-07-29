pub mod game_state;

use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::data::terrain::{Terrain, BackgroundCell};
use crate::shared::{PositionF32, pos, aabb_u32};

use super::GameClient;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, TryFromBytes, IntoBytes, Immutable)]
#[repr(u32)]
pub enum GameStateValue {
    #[default]
    Uninitialized,
    Game,
}

#[derive(Default, Copy, Clone)]
pub struct GameState {
    pub value: GameStateValue,
    pub scroll_view: bool,
    pub selection_start: Option<PositionF32>,
}

impl crate::store::StoreLoad for GameState {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.value);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut state = GameState::default();
        state.value = reader.try_read()?;
        Ok(state)
    }
}

fn paint_terrain(terrain: &mut Terrain) {
    terrain.paint_rect(BackgroundCell::Grass, aabb_u32(3, 3, 20, 12));
}

pub fn init(client: &mut GameClient) {
    let wd = &mut client.world_data;

    wd.reset();
    wd.initialize_terrain(32, 24);
    paint_terrain(&mut wd.data.terrain);

    wd.add_castle(pos(300.0, 300.0));
    wd.add_tower(pos(650.0, 300.0));
    wd.add_house(pos(850.0, 300.0));

    wd.add_knight(pos(300.0, 550.0));
    wd.add_knight(pos(300.0, 650.0));

    wd.compute_navigation();

    wd.data.common.view_offset = pos(-32.0, -32.0);
    wd.data.common.render_flags.set_update_view_offset();

    client.state.value = GameStateValue::Game;
}
