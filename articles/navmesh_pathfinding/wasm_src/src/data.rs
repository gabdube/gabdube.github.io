pub mod base;

pub mod assets;
use assets::Assets;

pub mod world;
use world::World;

pub mod terrain;
use terrain::Terrain;

#[cfg(feature="gui")] pub mod gui;
#[cfg(not(feature="gui"))] pub mod nogui;
#[cfg(not(feature="gui"))] pub use nogui as gui;
use gui::Gui;

pub mod debug;
use debug::DebugState;

use crate::shared::{PositionF32, SizeF32, pos};
use crate::store::StoreLoad;

const ANIMATION_INTERVAL: f64 = 1000.0 / 16.0; // 16fps


#[derive(Default, Copy, Clone)]
pub struct GlobalParams {
    pub time: f64,
    pub last_animation_tick: f64,
    pub time_delta: f32,

    pub flags: base::GameFlags,
    pub debug_flags: base::DebugFlags,
    
    pub mouse_position_old: PositionF32,
    pub mouse_position: PositionF32,
    pub view_offset: PositionF32,
    pub view_size: SizeF32,
    pub mouse_buttons: [base::ButtonState; 3],

    pub total_sprites: u32,
}

impl GlobalParams {
    pub fn primary_mouse_just_pressed(&self) -> bool { self.mouse_buttons[0].just_pressed() }
    pub fn middle_mouse_just_pressed(&self) -> bool { self.mouse_buttons[2].just_pressed() }
    pub fn middle_mouse_released(&self) -> bool { self.mouse_buttons[2].released() }
    pub fn mouse_moved(&self) -> bool {
        self.mouse_position_old.x != self.mouse_position.x || self.mouse_position_old.y != self.mouse_position.y
    }
    pub fn mouse_delta(&self) -> Option<PositionF32> {
        let delta_x = self.mouse_position_old.x - self.mouse_position.x;
        let delta_y = self.mouse_position_old.y - self.mouse_position.y;
        if delta_x != 0.0 || delta_y != 0.0 {
            Some(pos(delta_x, delta_y))
        } else {
            None
        }
    }
    
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
        self.globals.flags.set_update_terrain();
        self.globals.total_sprites = 0;
    }

    pub fn clear_sprites(&mut self) {
        self.world = World::default();
        self.globals.total_sprites = 0;
    }

    pub fn initialize_terrain(&mut self, width: u32, height: u32) {
        self.terrain.init(width, height);
        self.globals.flags.set_update_terrain();
    }

    pub fn prepare_update(&mut self, new_time: f64) {
        self.debug.clear();
        
        let global = &mut self.globals;
        global.time_delta = (new_time - global.time) as f32;
        global.time = new_time;

        // Can happen if the application was paused or hot reloaded.
        // In this case we set the delta to 0 for this frame so the game logic doesn't break.
        if global.time_delta > 1000.0 {
            global.time_delta = 0.0;
            global.last_animation_tick = new_time;
            global.mouse_position_old = global.mouse_position;
        }

        // Note: Sprite animation are computed at sprite generation in `output.render_sprites` 
        let delta = new_time - global.last_animation_tick;
        if delta > ANIMATION_INTERVAL {
            global.flags.set_update_animations();
            global.last_animation_tick = new_time;
        }

        self.gui.update_time(global.time_delta);
    }

    pub fn finalize_update(&mut self) {
        let g = &mut self.globals;
        g.mouse_buttons[0].flip();
        g.mouse_buttons[1].flip();
        g.mouse_buttons[2].flip();
        g.mouse_position_old = g.mouse_position;

        if self.gui.update() {
            g.flags.set_update_gui();
        }
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.globals.mouse_position = pos(x, y);
        self.gui.update_mouse_position(x, y);
    }

    pub fn update_mouse_buttons(&mut self, button: u8, pressed: bool) {
        let index = button as usize;
        if index < self.globals.mouse_buttons.len() {
            self.globals.mouse_buttons[index] = match pressed {
                true => base::ButtonState::JustPressed,
                false => base::ButtonState::JustReleased,
            };
        }

        self.gui.update_mouse_buttons(self.globals.mouse_position, button, pressed);
    }

    pub fn add_pawn(&mut self, position: PositionF32) {
        let idle = self.assets.atlas.pawn_idle;
        self.world.add_pawn(position, idle.animate());
        self.globals.total_sprites += 1;
    }

    pub fn add_house(&mut self, position: PositionF32) {
        let house = self.assets.atlas.house;
        self.world.add_house(position, house);
        self.globals.total_sprites += 1;
    }

    pub fn add_castle(&mut self, position: PositionF32) { 
        let castle = self.assets.atlas.castle;
        self.world.add_castle(position, castle);
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
        writer.write(&self.debug_flags);

        writer.write(&self.mouse_position_old);
        writer.write(&self.mouse_position);
        writer.write(&self.view_offset);
        writer.write(&self.view_size);
        
        writer.write(&self.total_sprites);
        
    }
    
    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut params = GlobalParams::default();
        params.flags = reader.try_read()?;
        params.debug_flags = reader.try_read()?;
        
        params.mouse_position_old = reader.try_read()?;
        params.mouse_position = reader.try_read()?;
        params.view_offset = reader.try_read()?;
        params.view_size = reader.try_read()?;

        params.total_sprites = reader.try_read()?;
       
        Ok(params)
    }
}
