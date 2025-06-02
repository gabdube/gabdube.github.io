pub mod base;

pub mod assets;
use assets::Assets;

pub mod world;
use world::World;

pub mod terrain;
use terrain::Terrain;

#[cfg(feature="gui")] pub mod gui;
#[cfg(not(feature="gui"))] pub mod nogui;
#[cfg(not(feature="gui"))] use nogui as gui;
use gui::Gui;

pub mod debug;
use debug::DebugState;

use crate::shared::{PositionF32, pos};
use crate::store::StoreLoad;

const ANIMATION_INTERVAL: f64 = 1000.0 / 16.0; // 16fps

#[derive(Default, Copy, Clone)]
pub struct GlobalParams {
    pub time: f64,
    pub last_animation_tick: f64,
    pub flags: base::GameFlags,
    pub total_sprites: u32,
    pub frame_delta: f32,
    pub mouse_position: PositionF32,
}

#[derive(Default)]
pub struct GameData {
    pub globals: GlobalParams,
    pub assets: Assets,
    pub world: World,
    pub terrain: Terrain,
    pub debug: DebugState,
    pub gui: Gui,
}

impl GameData {

    pub fn reset(&mut self) {
        self.world = World::default();
        self.terrain = Terrain::default();
    }

    pub fn initialize_terrain(&mut self, width: u32, height: u32) {
        self.terrain.init(width, height);
        self.globals.flags.set_update_terrain();
    }

    pub fn prepare_update(&mut self, new_time: f64) {
        self.debug.clear();
        
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

        self.gui.update_time(global.frame_delta);
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.globals.mouse_position = pos(x, y);
    }

    pub fn update_gui(&mut self) {
        if self.gui.update() {
            self.globals.flags.set_update_gui();
        }
    }

    pub fn add_pawn(&mut self, position: PositionF32) {
        let idle = self.assets.atlas.pawn_idle;
        self.world.add_pawn(position, idle.animate());
        self.globals.total_sprites += 1;
    }

}

impl StoreLoad for GameData {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        self.globals.store(writer);
        self.assets.store(writer);
        self.world.store(writer);
        self.terrain.store(writer);
        self.gui.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut data = GameData::default();
        data.globals = GlobalParams::load(reader)?;
        data.assets = Assets::load(reader)?;
        data.world = World::load(reader)?;
        data.terrain = Terrain::load(reader)?;
        data.gui = Gui::load(reader)?;

        data.gui.load_font(&data.assets)?;
        data.gui.load_style();

        Ok(data)
    }
}

impl StoreLoad for GlobalParams {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.flags);
        writer.write(&self.total_sprites);
        writer.write(&self.mouse_position);
    }
    
    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut params = GlobalParams::default();
        params.flags = reader.try_read()?;
        params.total_sprites = reader.try_read()?;
        params.mouse_position = reader.try_read()?;
        Ok(params)
    }
}
