use crate::state::{GameStateValue, GameInputType};
use crate::data::base::DebugFlags;
use super::GuiEvent;

pub(super) struct LeftPanelParams<'a> {
    pub events: &'a mut Vec<GuiEvent>,
    pub state: &'a mut GameStateValue,
    pub state_input: &'a mut GameInputType,
    pub panel_width: f32,
}

pub(super) struct PanelParams<'a> {
    pub events: &'a mut Vec<GuiEvent>,
    pub debug_flags: &'a mut DebugFlags,
    pub state_input: &'a mut GameInputType,
}

pub fn left_panel(ui: &mut egui::Ui, params: LeftPanelParams) {
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .exact_width(params.panel_width)
        .show_inside(ui, |ui| {
            ui.vertical(|ui| {
                let mut game_state_update = false;
                game_state_update |= ui.selectable_value(params.state, GameStateValue::Generation, "Generation").clicked();
                game_state_update |= ui.selectable_value(params.state, GameStateValue::Navigation, "Navigation").clicked();
                game_state_update |= ui.selectable_value(params.state, GameStateValue::Obstacles, "Obstacles").clicked();
                game_state_update |= ui.selectable_value(params.state, GameStateValue::FinalDemo, "Final Demo").clicked();

                if game_state_update {
                    *params.state_input = GameInputType::Select;
                    params.events.push(GuiEvent::GameStateValueChanged(*params.state));
                    params.events.push(GuiEvent::SetInputType(GameInputType::Select));
                }
            });
        });
}

pub fn right_panel<F>(ui: &mut egui::Ui, width: f32, callback: F) 
    where F: FnOnce(&mut egui::Ui)
{
    egui::SidePanel::right("right_panel")
        .resizable(false)
        .exact_width(width)
        .show_separator_line(false)
        .show_inside(ui, callback);
}

pub fn generation_panel(ui: &mut egui::Ui, params: PanelParams) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if ui.button("Reset World").clicked() {
                params.events.push(GuiEvent::ResetWorld);
            }
            
            let mut input_update = false;
            input_update |= ui.selectable_value(params.state_input, GameInputType::Select, "Select").clicked();
            input_update |= ui.selectable_value(params.state_input, GameInputType::Delete, "Delete").clicked();
            input_update |= ui.selectable_value(params.state_input, GameInputType::PlacePawn, "Add Pawn").clicked();
            input_update |= ui.selectable_value(params.state_input, GameInputType::PlaceCastle, "Add Castle").clicked();
            input_update |= ui.selectable_value(params.state_input, GameInputType::PlaceHouse, "Add House").clicked();
            if input_update {
                params.events.push(GuiEvent::SetInputType(*params.state_input));
            }
        });
        ui.separator();
        bitflag_checkbox(ui, params.events, "Show navmesh", params.debug_flags, DebugFlags::SHOW_NAVMESH);
    });
}

pub fn navigation_panel(ui: &mut egui::Ui, params: PanelParams) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if ui.button("Reset Pawn").clicked() {
                params.events.push(GuiEvent::ResetPawnPosition);
            }
        });
        ui.separator();
        bitflag_checkbox(ui, params.events, "Show navmesh", params.debug_flags, DebugFlags::SHOW_NAVMESH);
        bitflag_checkbox(ui, params.events, "Show hovered triangle", params.debug_flags, DebugFlags::SHOW_HOVERED_TRIANGLE);
        bitflag_checkbox(ui, params.events, "Show cell centers", params.debug_flags, DebugFlags::SHOW_CELL_CENTERS);
        bitflag_checkbox(ui, params.events, "Show path", params.debug_flags, DebugFlags::SHOW_PATH);
    });
}

pub fn obstacles_panel(ui: &mut egui::Ui, params: PanelParams) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if ui.button("Reset Pawn").clicked() {
                params.events.push(GuiEvent::ResetPawnPosition);
            }
        });
        ui.separator();
        bitflag_checkbox(ui, params.events, "Show navmesh", params.debug_flags, DebugFlags::SHOW_NAVMESH);
        bitflag_checkbox(ui, params.events, "Show collisions box", params.debug_flags, DebugFlags::SHOW_COLLISION_BOXES);
        bitflag_checkbox(ui, params.events, "Show blocked cells", params.debug_flags, DebugFlags::SHOW_BLOCKED_CELLS);
        bitflag_checkbox(ui, params.events, "Show path", params.debug_flags, DebugFlags::SHOW_PATH);
    });
}

pub fn final_panel(ui: &mut egui::Ui, params: PanelParams) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if ui.button("Reset World").clicked() {
                params.events.push(GuiEvent::ResetWorld);
            }
            if ui.button("Reset Pawn").clicked() {
                params.events.push(GuiEvent::ResetPawnPosition);
            }
        });

        ui.horizontal(|ui| {
            let mut input_update = false;
            input_update |= ui.selectable_value(params.state_input, GameInputType::Select, "Select").clicked();
            input_update |= ui.selectable_value(params.state_input, GameInputType::Delete, "Delete").clicked();
            input_update |= ui.selectable_value(params.state_input, GameInputType::PlacePawn, "Add Pawn").clicked();
            input_update |= ui.selectable_value(params.state_input, GameInputType::PlaceCastle, "Add Castle").clicked();
            input_update |= ui.selectable_value(params.state_input, GameInputType::PlaceHouse, "Add House").clicked();
            if input_update {
                params.events.push(GuiEvent::SetInputType(*params.state_input));
            }
        });
    });
}

fn bitflag_checkbox(
    ui: &mut egui::Ui,
    events: &mut Vec<GuiEvent>,
    value: &str,
    flags: &mut DebugFlags,
    mask: u32
) {
    let mut check_value = flags.0 & mask > 0;
    if ui.checkbox(&mut check_value, value).changed() {
        if check_value {
            flags.0 |= mask;
        } else {
            flags.0 &= !mask;
        }
        events.push(GuiEvent::SetDebugFlags(*flags));
    }   
}
