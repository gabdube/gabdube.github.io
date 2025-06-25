pub mod base;

pub mod assets;
use assets::Assets;

pub mod world;
use world::World;

pub mod terrain;
use terrain::Terrain;

pub mod behaviour;
use behaviour::BehaviourState;

#[cfg(feature="gui")] pub mod gui;
#[cfg(not(feature="gui"))] pub mod nogui;
#[cfg(not(feature="gui"))] pub use nogui as gui;
use gui::Gui;

pub mod navigation;
use navigation::NavigationState;

pub mod debug;
use debug::DebugState;

use crate::shared::{PositionF32, SizeF32, pos};
use crate::store::StoreLoad;

const ANIMATION_INTERVAL: f64 = 1000.0 / 16.0; // 16fps


#[derive(Default, Copy, Clone)]
pub struct CommonParams {
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

impl CommonParams {
    pub fn primary_mouse_just_pressed(&self) -> bool { self.mouse_buttons[0].just_pressed() }
    pub fn secondary_mouse_just_pressed(&self) -> bool { self.mouse_buttons[1].just_pressed() }
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
    pub common: CommonParams,
    pub assets: Assets,
    pub terrain: Terrain,
    pub navigation: NavigationState,
    pub behaviours: BehaviourState,
    pub debug: DebugState,
    pub gui: Gui,
}

#[derive(Default)]
pub struct GameWorldData {
    pub world: World,
    pub data: GameData,
}

impl GameWorldData {

    pub fn reset(&mut self) {
        self.world = World::default();

        let data = &mut self.data;
        data.terrain = Terrain::default();
        data.navigation.clear();
        data.common.flags.set_update_terrain();
        data.common.total_sprites = 0;
    }

    pub fn clear_sprites(&mut self) {
        self.world = World::default();

        let data = &mut self.data;
        data.navigation.clear();
        data.common.total_sprites = 0;
    }

    pub fn initialize_terrain(&mut self, width: u32, height: u32) {
        self.data.terrain.init(width, height);
        self.data.common.flags.set_update_terrain();
    }

    pub fn compute_navigation(&mut self) {
        NavigationState::rebuild_navmesh(self);
    }

    pub fn prepare_update(&mut self, new_time: f64) {
        let data = &mut self.data;
        
        data.debug.clear();
        
        let global = &mut data.common;
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

        data.gui.update_time(global.time_delta);
    }

    pub fn finalize_update(&mut self) {
        let c = &mut self.data.common;
        c.mouse_buttons[0].flip();
        c.mouse_buttons[1].flip();
        c.mouse_buttons[2].flip();
        c.mouse_position_old = c.mouse_position;
    }

    pub fn update_gui(&mut self) {
        if self.data.gui.update() {
            self.data.common.flags.set_update_gui();
        }
    }

    pub fn run_behaviours(&mut self) {
        BehaviourState::run(self);
    }

    pub fn generate_debug_info(&mut self) {
        let data = &mut self.data;
        let debug = &mut data.debug;
        let debug_flags = data.common.debug_flags;

        if debug_flags.show_navmesh() {
            data.navigation.debug_navmesh(debug, debug_flags.show_cell_centers());
        }

        if debug_flags.show_blocked_cells() {
            data.navigation.debug_blocked_cells(debug);
        }

        if debug_flags.show_pathfinding_graph() {
            data.navigation.debug_pathfinding_graph(debug);
        }
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        let data = &mut self.data;
        data.common.mouse_position = pos(x, y);
        data.gui.update_mouse_position(x, y);
    }

    pub fn update_mouse_buttons(&mut self, button: u8, pressed: bool) {
        let data = &mut self.data;
        let index = button as usize;
        if index < data.common.mouse_buttons.len() {
            data.common.mouse_buttons[index] = match pressed {
                true => base::ButtonState::JustPressed,
                false => base::ButtonState::JustReleased,
            };
        }

        data.gui.update_mouse_buttons(data.common.mouse_position, button, pressed);
    }

    pub fn delete_sprite_at_position(&mut self, position: PositionF32) {
        if self.world.delete_sprite_at_position(position) {
            self.data.common.total_sprites -= 1;
        }
    }

    pub fn add_pawn(&mut self, position: PositionF32) {
        let idle = self.data.assets.atlas.pawn_idle;
        self.world.add_pawn(position, idle.animate());
        self.data.common.total_sprites += 1;
    }

    pub fn add_house(&mut self, position: PositionF32) {
        let house = self.data.assets.atlas.house;
        self.world.add_house(position, house);
        self.data.common.total_sprites += 1;
    }

    pub fn add_castle(&mut self, position: PositionF32) { 
        let castle = self.data.assets.atlas.castle;
        self.world.add_castle(position, castle);
        self.data.common.total_sprites += 1;
    }

}

impl StoreLoad for GameWorldData {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        self.world.store(writer);

        let data = &mut self.data;
        data.common.store(writer);
        data.assets.store(writer);
        data.terrain.store(writer);
        data.navigation.store(writer);
        data.behaviours.store(writer);
        data.gui.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut world_data = GameWorldData::default();

        world_data.world = World::load(reader)?;

        let data = &mut world_data.data;
        data.common = CommonParams::load(reader)?;
        data.assets = Assets::load(reader)?;
        data.terrain = Terrain::load(reader)?;
        data.navigation = NavigationState::load(reader)?;
        data.behaviours = BehaviourState::load(reader)?;
        data.gui = Gui::load(reader)?;

        data.gui.load_font(&data.assets)?;
        data.gui.load_style();

        Ok(world_data)
    }
}

impl StoreLoad for CommonParams {
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
        let mut params = CommonParams::default();
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
