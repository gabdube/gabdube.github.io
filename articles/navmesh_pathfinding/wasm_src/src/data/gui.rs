#![cfg(feature="gui")]
mod components;

use crate::shared::PositionF32;
use crate::store::StoreLoad;
use crate::data::base::DebugFlags;
use crate::state::{GameStateValue, GameInputType};
use crate::GameClientInit;

#[derive(Copy, Clone)]
pub enum GuiEvent {
    GameStateValueChanged(GameStateValue),
    SetDebugFlags(DebugFlags),
    SetInputType(GameInputType),
    ResetWorld,
    ResetPawnPosition,
}

/// Egui wrapper
pub struct Gui {
    ctx: egui::Context,
    input: Box<egui::RawInput>,
    output: Box<egui::FullOutput>,
    height: f32,
    pixel_per_point: f32,
    max_texture_size: u32,
    view: [f32; 4],
    game_state: GameStateValue,
    game_input: GameInputType,
    debug_flags: DebugFlags,
    events: Vec<GuiEvent>,
    force_repaint: bool,
}

impl Gui {

    pub fn init(&mut self, init: &GameClientInit, assets: &crate::data::Assets) -> Result<(), crate::Error> {
        let input = &mut self.input;

        let min = egui::Pos2 { x: 0.0, y: init.view_size.height - self.height };
        let size = egui::Vec2 { x: init.view_size.width, y: self.height };
        input.screen_rect = Some(egui::Rect::from_min_size(min, size));

        input.max_texture_side = Some(init.max_texture_size as usize);
        input.time = Some(0.0);
        input.system_theme = Some(egui::Theme::Dark);

        self.view = [min.x, min.y, size.x, size.y];
        self.max_texture_size = init.max_texture_size;

        self.load_font(assets)?;
        self.load_style();

        Ok(())
    }

    // UI generation happens here
    pub fn update(&mut self) -> bool {
        let input = egui::RawInput::take(&mut self.input);
        self.ctx.begin_pass(input);

        let width = self.view[2];
        let left_panel_width = 140.0;

        egui::CentralPanel::default().show(&self.ctx, |ui| {
            components::left_panel(ui, components::LeftPanelParams {
                events: &mut self.events,
                state: &mut self.game_state,
                state_input: &mut self.game_input,
                panel_width: left_panel_width,
            });

            let params = components::PanelParams {
                events: &mut self.events,
                debug_flags: &mut self.debug_flags,
                state_input: &mut self.game_input,
            };

            components::right_panel(ui, width-left_panel_width, |ui| {
                match self.game_state {
                    GameStateValue::Generation => components::generation_panel(ui, params),
                    GameStateValue::Navigation => components::navigation_panel(ui, params),
                    GameStateValue::Obstacles => components::obstacles_panel(ui, params),
                    GameStateValue::FinalDemo => components::final_panel(ui, params),
                    _ => {}
                }
            })
        });

        *self.output = self.ctx.end_pass();

        self.ctx.has_requested_repaint() || ::std::mem::take(&mut self.force_repaint)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let min = egui::Pos2 { x: 0.0, y: (height as f32) - self.height };
        let size = egui::Vec2 { x: (width as f32), y: self.height };
        self.input.screen_rect = Some(egui::Rect::from_min_size(min, size));
        self.view = [min.x, min.y, size.x, size.y];
        self.force_repaint = true;
    }

    pub fn set_state(&mut self, state: GameStateValue, input: GameInputType) {
        self.game_state = state;
        self.game_input = input;
        self.force_repaint = true;
    }

    pub fn set_debug_flags(&mut self, flags: DebugFlags) {
        self.debug_flags = flags;
        self.force_repaint = true;
    }

    pub fn position_outside_gui(&self, position: PositionF32) -> bool {
        position.y < self.view[1]
    }

    pub fn position_inside_gui(&self, position: PositionF32) -> bool {
        position.y >= self.view[1]
    }

    pub fn events(&mut self) -> Vec<GuiEvent> {
        let cloned;
        if self.events.len() > 0 {
            cloned = self.events.clone();
        } else {
            cloned = Vec::new();
        }
        self.events.clear();
        cloned
    }

    pub(super) fn update_time(&mut self, delta: f32) {
        if let Some(time) = self.input.time.as_mut() {
            *time += (delta as f64) / 2000.0;
        }
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        if y >= self.view[1] {
            self.input.events.push(egui::Event::MouseMoved(egui::Vec2 { x, y }));
            self.input.focused = true;
        }
    }

    pub fn update_mouse_buttons(&mut self, position: PositionF32, button: u8, pressed: bool) {
        use egui::{Event, Modifiers, PointerButton};
        
        let pos = egui::Pos2 { x: position.x, y: position.y };
        let button = match button {
            0 => Some(PointerButton::Primary),
            1 => Some(PointerButton::Secondary),
            2 => Some(PointerButton::Middle),
            _ => None
        };

        if let Some(button) = button {
            self.input.events.push(Event::PointerButton { pos, button, pressed, modifiers: Modifiers::default() });
        }
    }

    pub fn update_keys(&mut self, key_name: &str, pressed: bool) {
        use egui::{Event, Modifiers, Key};
        let key = Key::from_name(key_name);

        if let Some(key) = key {
            self.input.events.push(Event::Key { key, physical_key: None, pressed, repeat: false, modifiers: Modifiers::default() })
        }
    }

    pub fn texture_delta(&mut self) -> egui::TexturesDelta {
        ::std::mem::take(&mut self.output.textures_delta)
    }

    pub fn tesselate(&mut self) -> Vec<egui::ClippedPrimitive> {
        let shapes = std::mem::take(&mut self.output.shapes);
        self.ctx.tessellate(shapes, self.pixel_per_point)
    }

    pub fn load_font(&mut self, assets: &crate::data::Assets) -> Result<(), crate::Error>  {
        let mut fonts = egui::FontDefinitions::default();

        let font_name = "firacode".to_string();
        let font_data;
        match assets.fonts.get(&font_name) {
            Some(data) => { font_data = data; },
            None => { return Err(assets_err!("Missing font source for font \"firacode\"")); }
        }

        fonts.font_data.insert(font_name.clone(), std::sync::Arc::new(  egui::FontData::from_owned(font_data.clone()) ) );
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().push(font_name);

        self.ctx.set_fonts(fonts);

        Ok(())
    }

    pub fn load_style(&mut self) {
        use egui::{TextStyle, FontId, FontFamily, Color32};

        self.ctx.style_mut(|style| {
            style.visuals.panel_fill = Color32::from_rgb(48, 43, 40);
            style.visuals.override_text_color = Some(Color32::from_rgba_unmultiplied(224, 224, 224, 255));
            style.text_styles.insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
            style.text_styles.insert(TextStyle::Button, FontId::new(15.0, FontFamily::Proportional));
        });
    }
}

impl StoreLoad for Gui {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.height);
        writer.write(&self.pixel_per_point);
        writer.write(&self.max_texture_size);
        writer.write(&self.view);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut gui = Gui::default();
        gui.height = reader.try_read()?;
        gui.pixel_per_point = reader.try_read()?;
        gui.max_texture_size = reader.try_read()?;
        gui.view = reader.try_read()?;

        let input = &mut gui.input;
        input.max_texture_side = Some(gui.max_texture_size as usize);
        input.time = Some(0.0);
        input.system_theme = Some(egui::Theme::Dark);

        let min = egui::Pos2 { x: gui.view[0], y: gui.view[1] };
        let size = egui::Vec2 { x: gui.view[2], y: gui.view[3] };
        input.screen_rect = Some(egui::Rect::from_min_size(min, size));

        Ok(gui)
    }
}

impl Default for Gui {

    fn default() -> Self {
        Gui {
            ctx: egui::Context::default(),
            input: Box::default(),
            output: Box::default(),
            height: 300.0,
            pixel_per_point: 1.0,
            max_texture_size: 2048,
            view: [0.0; 4],
            game_state: GameStateValue::Uninitialized,
            game_input: GameInputType::Select,
            debug_flags: DebugFlags::default(),
            events: Vec::new(),
            force_repaint: true,
        }
    }

}
