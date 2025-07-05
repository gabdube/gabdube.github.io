pub mod base;

pub mod terrain;
use terrain::Terrain;

use crate::shared::{PositionF32, SizeF32, pos};

pub struct CommonParams {
    // Flags
    pub render_flags: base::RenderFlags,

    // Global parameters
    pub view_size: SizeF32,
    pub view_offset: PositionF32,
    pub zoom: f32,

    // Inputs
    pub mouse_position_old: PositionF32,
    pub mouse_position: PositionF32,
    pub mouse_position_gui: PositionF32,
    pub mouse_buttons: [base::ButtonState; 3],
}

impl CommonParams {
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
}

#[derive(Default)]
pub struct GameData {
    pub common: CommonParams,
    pub terrain: Terrain,
}

#[derive(Default)]
pub struct GameWorldData {
    //pub world: World,
    pub data: GameData,
}

impl GameWorldData {
    pub fn reset(&mut self) {
        let data = &mut self.data;
        data.terrain = Terrain::default();
    }

    pub fn initialize_terrain(&mut self, width: u32, height: u32) {
        self.data.terrain.init(width, height);
        self.data.common.render_flags.set_update_terrain();
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

    pub fn finalize_update(&mut self) {
        let c = &mut self.data.common;
        c.mouse_buttons[0].flip();
        c.mouse_buttons[1].flip();
        c.mouse_buttons[2].flip();
        c.mouse_position_old = c.mouse_position;
    }
}

impl crate::store::StoreLoad for GameWorldData {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        let data = &mut self.data;
        data.common.store(writer);
        data.terrain.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut world_data = GameWorldData::default();

        let data = &mut world_data.data;
        data.common = CommonParams::load(reader)?;
        data.terrain = Terrain::load(reader)?;

        Ok(world_data)
    }
}

impl crate::store::StoreLoad for CommonParams {

    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.render_flags);

        writer.write(&self.view_size);
        writer.write(&self.view_offset);
        writer.write(&self.zoom);

        writer.write(&self.mouse_position_old);
        writer.write(&self.mouse_position);
        writer.write(&self.mouse_position_gui);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut params = CommonParams::default();

        params.render_flags = reader.try_read()?;

        params.view_size = reader.try_read()?;
        params.view_offset = reader.try_read()?;
        params.zoom = reader.try_read()?;

        params.mouse_position_old = reader.try_read()?;
        params.mouse_position = reader.try_read()?;
        params.mouse_position_gui = reader.try_read()?;

        Ok(params)
    }
}



impl Default for CommonParams {
    fn default() -> Self {
        CommonParams {
            render_flags: base::RenderFlags(0),
            view_size: SizeF32 { width: 0.0, height: 0.0 },
            view_offset: pos(0.0, 0.0),
            zoom: 1.0,

            mouse_position_old: pos(0.0, 0.0),
            mouse_position: pos(0.0, 0.0),
            mouse_position_gui: pos(0.0, 0.0),
            mouse_buttons: [base::ButtonState::default(); 3],
        }
    }
}
