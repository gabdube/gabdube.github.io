pub mod base;
pub mod sprites;

pub mod assets;
use assets::Assets;

pub mod debug;
use debug::DebugState;

pub mod terrain;
use terrain::Terrain;

pub mod navigation;
use navigation::NavigationState;

pub mod behaviour;
use behaviour::BehaviourState;

pub mod world;
use world::World;

mod extra;

use fnv::FnvHashMap;
use crate::shared::{pos, PositionF32, SizeF32};

const ANIMATION_INTERVAL: f64 = 1000.0 / 16.0; // 16fps

pub struct CommonParams {
    // Time
    pub time: f64,
    pub last_animation_tick: f64,
    pub time_delta: f32,

    // Flags
    pub render_flags: base::RenderFlags,
    pub debug_flags: base::DebugFlags,

    // Global parameters
    pub view_size: SizeF32,
    pub view_offset: PositionF32,
    pub zoom: f32,
    pub total_sprites: u32,

    // Inputs
    pub mouse_position_old: PositionF32,
    pub mouse_position: PositionF32,
    pub mouse_position_gui: PositionF32,
    pub mouse_buttons: [base::ButtonState; 3],
    pub keys: FnvHashMap<base::keys::Key, base::ButtonState>,
}

impl CommonParams {
    pub fn primary_mouse_just_pressed(&self) -> bool { self.mouse_buttons[0].just_pressed() }
    pub fn primary_mouse_just_released(&self) -> bool { self.mouse_buttons[0].just_released() }
    pub fn secondary_mouse_just_pressed(&self) -> bool { self.mouse_buttons[1].just_pressed() }
    pub fn middle_mouse_just_pressed(&self) -> bool { self.mouse_buttons[2].just_pressed() }
    pub fn middle_mouse_released(&self) -> bool { self.mouse_buttons[2].released() }

    pub fn mouse_delta(&self) -> Option<PositionF32> {
        let delta_x = self.mouse_position_old.x - self.mouse_position.x;
        let delta_y = self.mouse_position_old.y - self.mouse_position.y;
        if delta_x != 0.0 || delta_y != 0.0 {
            Some(pos(delta_x, delta_y))
        } else {
            None
        }
    }

    pub fn key_just_pressed(&self, key: base::keys::Key) -> bool {
        self.keys.get(&key).copied()
            .unwrap_or(base::ButtonState::Released)
            .just_pressed()
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
        data.common.render_flags.set_update_terrain();
        data.common.total_sprites = 0;
    }

    pub fn initialize_terrain(&mut self, width: u32, height: u32) {
        self.data.terrain.init(width, height);
        self.data.common.render_flags.set_update_terrain();
    }

    pub fn compute_navigation(&mut self) {
        NavigationState::rebuild_navmesh(self);
    }

    pub fn run_behaviours(&mut self) {
        BehaviourState::run(self);
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        let data = &mut self.data;
        let zoom = 1.0 / data.common.zoom;
        data.common.mouse_position_gui = pos(x, y);
        data.common.mouse_position = pos(x*zoom, y*zoom);
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
    }

    pub fn update_key(&mut self, key: &str, pressed: bool) {
        let button_state = match pressed {
            true => base::ButtonState::JustPressed,
            false => base::ButtonState::JustReleased,
        };

        if let Some(key) = base::keys::Key::from_str(key) {
            self.data.common.keys.insert(key, button_state);
        }
    }

    /// Mouse position in world coordinates. Ie: relative to the world view offset
    pub fn world_mouse_position(&self) -> PositionF32 {
        let mouse_position = self.data.common.mouse_position;
        let view_offset = self.data.common.view_offset;
        mouse_position - view_offset
    }

    pub fn prepare_update(&mut self, new_time: f64) {
        let data = &mut self.data;
        
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
            global.render_flags.set_update_animations();
            global.last_animation_tick = new_time;
        }

        // Debug state reset
        data.debug.clear();
    }

    pub fn handle_global_inputs(&mut self) {
        let common = &mut self.data.common;
        if common.key_just_pressed(base::keys::KEY_DIGIT_1) {
            common.debug_flags.toggle(base::DebugFlags::DEBUG_WORLD_GRID);
        }

        if common.key_just_pressed(base::keys::KEY_DIGIT_2) {
            common.debug_flags.toggle(base::DebugFlags::DEBUG_DISPLAY_GRID);
        }
    }

    pub fn global_updates(&mut self) {
        let debug = self.data.common.debug_flags;
        if debug.debug_world_grid() {
            extra::debug_world_grid(&mut self.data);
        }
        if debug.debug_display_grid() {
            extra::debug_display_grid(&mut self.data);
        }
    }

    pub fn finalize_update(&mut self) {
        let c = &mut self.data.common;
        c.mouse_buttons[0].flip();
        c.mouse_buttons[1].flip();
        c.mouse_buttons[2].flip();
        c.mouse_position_old = c.mouse_position;

        for key_state in c.keys.values_mut() {
            key_state.flip();
        }
    }

    pub fn add_castle(&mut self, position: PositionF32) { 
        let castle = self.data.assets.atlas.castle;
        self.world.add_castle(position, castle);
        self.data.common.total_sprites += 1;
    }

    pub fn add_tower(&mut self, position: PositionF32) { 
        let tower = self.data.assets.atlas.tower;
        self.world.add_tower(position, tower);
        self.data.common.total_sprites += 1;
    }

    pub fn add_house(&mut self, position: PositionF32) { 
        let house = self.data.assets.atlas.house;
        self.world.add_house(position, house);
        self.data.common.total_sprites += 1;
    }

    pub fn add_knight(&mut self, position: PositionF32) {
        self.world.add_knight(position);
        self.data.common.total_sprites += 1;
    }
}

impl crate::store::StoreLoad for GameWorldData {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        self.world.store(writer);
        
        let data = &mut self.data;
        data.common.store(writer);
        data.assets.store(writer);
        data.navigation.store(writer);
        data.terrain.store(writer);
        data.behaviours.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut world_data = GameWorldData::default();

        world_data.world = World::load(reader)?;

        let data = &mut world_data.data;
        data.common = CommonParams::load(reader)?;
        data.assets = Assets::load(reader)?;
        data.navigation = NavigationState::load(reader)?;
        data.terrain = Terrain::load(reader)?;
        data.behaviours = BehaviourState::load(reader)?;

        Ok(world_data)
    }
}

impl crate::store::StoreLoad for CommonParams {

    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.render_flags);
        writer.write(&self.debug_flags);

        writer.write(&self.view_size);
        writer.write(&self.view_offset);
        writer.write(&self.zoom);
        writer.write(&self.total_sprites);

        writer.write(&self.mouse_position_old);
        writer.write(&self.mouse_position);
        writer.write(&self.mouse_position_gui);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut params = CommonParams::default();

        params.render_flags = reader.try_read()?;
        params.debug_flags = reader.try_read()?;

        params.view_size = reader.try_read()?;
        params.view_offset = reader.try_read()?;
        params.zoom = reader.try_read()?;
        params.total_sprites = reader.try_read()?;

        params.mouse_position_old = reader.try_read()?;
        params.mouse_position = reader.try_read()?;
        params.mouse_position_gui = reader.try_read()?;

        Ok(params)
    }
}


impl Default for CommonParams {
    fn default() -> Self {
        CommonParams {
            time: 0.0,
            last_animation_tick: 0.0,
            time_delta: 0.0,

            render_flags: base::RenderFlags(0),
            debug_flags: base::DebugFlags(0),

            view_size: SizeF32 { width: 0.0, height: 0.0 },
            view_offset: pos(0.0, -1.0),
            zoom: 1.0,
            total_sprites: 0,

            mouse_position_old: pos(0.0, 0.0),
            mouse_position: pos(0.0, 0.0),
            mouse_position_gui: pos(0.0, 0.0),
            mouse_buttons: [base::ButtonState::default(); 3],
            keys: FnvHashMap::default()
        }
    }
}
