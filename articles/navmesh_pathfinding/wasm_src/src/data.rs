pub mod base;

pub mod assets;
use assets::Assets;

pub mod world;
use world::World;

use crate::shared::PositionF32;
use crate::store::StoreLoad;

const ANIMATION_INTERVAL: f64 = 1000.0 / 16.0; // 16fps

#[derive(Default, Copy, Clone)]
pub struct GlobalParams {
    pub time: f64,
    pub last_animation_tick: f64,
    pub flags: base::GameFlags,
    pub total_sprites: u32,
    pub frame_delta: f32,
}

pub struct GameData {
    pub globals: GlobalParams,
    pub assets: Assets,
    pub world: World,
}

impl GameData {

    pub fn update_timing(&mut self, new_time: f64) {
        let global = &mut self.globals;
        global.frame_delta = (new_time - global.time) as f32;
        global.time = new_time;

        // Can happen if the application was paused or hot reloaded.
        // In this case we set the delta to 0 for this frame so the game logic doesn't break.
        if global.frame_delta > 1000.0 {
            global.frame_delta = 0.0;
            global.last_animation_tick = new_time;
        }

        // Note: Sprite animation are computed at sprite generation in `output.render_sprites` 
        let delta = new_time - global.last_animation_tick;
        if delta > ANIMATION_INTERVAL {
            global.flags.set_update_animations();
            global.last_animation_tick = new_time;
        }
    }

    pub fn add_pawn(&mut self, position: PositionF32) {
        let idle = self.assets.atlas.pawn_idle;
        self.world.add_pawn(position, idle.animate());
        self.globals.total_sprites += 1;
    }

}

impl Default for GameData {
    fn default() -> Self {
        GameData {
            globals: GlobalParams::default(),
            assets: Assets::default(),
            world: World::default(),
        }
    }
}

impl StoreLoad for GameData {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        self.globals.store(writer);
        self.assets.store(writer);
        self.world.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut data = GameData::default();
        data.globals = GlobalParams::load(reader)?;
        data.assets = Assets::load(reader)?;
        data.world = World::load(reader)?;
        Ok(data)
    }
}

impl StoreLoad for GlobalParams {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.flags);
        writer.write(&self.total_sprites);
    }
    
    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut params = GlobalParams::default();
        params.flags = reader.try_read()?;
        params.total_sprites = reader.try_read()?;
        Ok(params)
    }
}
