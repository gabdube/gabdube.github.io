use crate::state::{GameStateValue, GameInputType};
use crate::data::base::DebugFlags;
use super::GuiEvent;

pub(super) struct LeftPanelParams<'a> {
    pub events: &'a mut Vec<GuiEvent>,
    pub state: &'a mut GameStateValue,
    pub state_input: &'a mut GameInputType,
    pub debug_flags: &'a mut DebugFlags,
    pub panel_width: f32,
}

pub(super) struct PanelParams<'a> {
    pub events: &'a mut Vec<GuiEvent>,
    pub debug_flags: &'a mut DebugFlags,
    pub state_input: &'a mut GameInputType,
    pub gui_width: f32,
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
                game_state_update |= ui.selectable_value(params.state, GameStateValue::Pathfinding, "Pathfinding").clicked();

                if game_state_update {
                    match params.state {
                        GameStateValue::Generation => {
                            params.events.push(GuiEvent::SetDebugFlags(params.debug_flags.and(DebugFlags::SHOW_NAVMESH)));
                        },
                        GameStateValue::Navigation => {
                            let flags = DebugFlags::SHOW_NAVMESH | DebugFlags::SHOW_CELL_CENTERS |
                                        DebugFlags::SHOW_TRIANGLE_LOOKUP | DebugFlags::SHOW_TRIANGLE_LOOKUP_PATH;
                            params.events.push(GuiEvent::SetDebugFlags(params.debug_flags.and(flags)));
                        },
                        GameStateValue::Pathfinding => {
                            let flags = DebugFlags::SHOW_NAVMESH | DebugFlags::SHOW_PATHFINDING_GRAPH |
                                        DebugFlags::SHOW_PATH_ROUGH | DebugFlags::SHOW_PATH;
                            params.events.push(GuiEvent::SetDebugFlags(params.debug_flags.and(flags)));
                        },
                        _ => {},
                    }

                    *params.state_input = GameInputType::Select;
                    params.events.push(GuiEvent::SetInputType(GameInputType::Select));

                    params.events.push(GuiEvent::GameStateValueChanged(*params.state));
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
        if params.gui_width < 700.0 {
            let mut input_update = false;

            ui.horizontal(|ui| {
                if ui.button("Reset World").clicked() {
                    params.events.push(GuiEvent::ResetWorld);
                }
            });

            ui.horizontal(|ui| {
                input_update |= ui.selectable_value(params.state_input, GameInputType::Select, "Select").clicked();
                input_update |= ui.selectable_value(params.state_input, GameInputType::Delete, "Delete").clicked();
                input_update |= ui.selectable_value(params.state_input, GameInputType::PlacePawn, "Add Pawn").clicked();
            });

            ui.horizontal(|ui| {
                input_update |= ui.selectable_value(params.state_input, GameInputType::PlaceCastle, "Add Castle").clicked();
                input_update |= ui.selectable_value(params.state_input, GameInputType::PlaceHouse, "Add House").clicked();
            });

            if input_update {
                params.events.push(GuiEvent::SetInputType(*params.state_input));
            }
        } else {
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
        }

      
        ui.separator();
        bitflag_checkbox(ui, params.events, "Show navmesh", params.debug_flags, DebugFlags::SHOW_NAVMESH, 0, DebugFlags::SHOW_CELL_CENTERS);
    });
}

pub fn navigation_panel(ui: &mut egui::Ui, params: PanelParams) {
    ui.vertical(|ui| {
        bitflag_checkbox(ui, params.events, "Show navmesh", params.debug_flags, DebugFlags::SHOW_NAVMESH, 0, DebugFlags::SHOW_CELL_CENTERS);
        bitflag_checkbox(ui, params.events, "Show cell centers", params.debug_flags, DebugFlags::SHOW_CELL_CENTERS, DebugFlags::SHOW_NAVMESH, 0);
        bitflag_checkbox(ui, params.events, "Show blocked cells", params.debug_flags, DebugFlags::SHOW_BLOCKED_CELLS, 0, 0);
        bitflag_checkbox(ui, params.events, "Show triangle lookup", params.debug_flags, DebugFlags::SHOW_TRIANGLE_LOOKUP, 0, DebugFlags::SHOW_TRIANGLE_LOOKUP_PATH);
        bitflag_checkbox(ui, params.events, "Debug triangle lookup", params.debug_flags, DebugFlags::SHOW_TRIANGLE_LOOKUP_PATH, DebugFlags::SHOW_TRIANGLE_LOOKUP, 0);
    });
}

pub fn pathfinding_panel(ui: &mut egui::Ui, params: PanelParams) {
    ui.vertical(|ui| {
        bitflag_checkbox(ui, params.events, "Show navmesh", params.debug_flags, DebugFlags::SHOW_NAVMESH, 0, DebugFlags::SHOW_CELL_CENTERS);
        bitflag_checkbox(ui, params.events, "Show pathfinding graph", params.debug_flags, DebugFlags::SHOW_PATHFINDING_GRAPH, 0, 0);
        bitflag_checkbox2(ui, params.events, "Debug pathfinding", params.debug_flags, DebugFlags::SHOW_PATH_ROUGH, DebugFlags::SHOW_PATH | DebugFlags::SHOW_PATH_FUNNEL);
        bitflag_checkbox2(ui, params.events, "Debug pathfinding funnel", params.debug_flags, DebugFlags::SHOW_PATH_FUNNEL, DebugFlags::SHOW_PATH | DebugFlags::SHOW_PATH_ROUGH);
        bitflag_checkbox2(ui, params.events, "Debug pathfinding (smoothed)", params.debug_flags, DebugFlags::SHOW_PATH, DebugFlags::SHOW_PATH_ROUGH | DebugFlags::SHOW_PATH_FUNNEL);
    });
}

fn bitflag_checkbox(
    ui: &mut egui::Ui,
    events: &mut Vec<GuiEvent>,
    value: &str,
    flags: &mut DebugFlags,
    mask: u32,
    extra_set: u32,
    extra_remove: u32,
) {
    let mut check_value = flags.0 & mask > 0;
    if ui.checkbox(&mut check_value, value).changed() {
        if check_value {
            flags.0 |= mask | extra_set;
        } else {
            flags.0 &= !(mask | extra_remove);
        }
        events.push(GuiEvent::SetDebugFlags(*flags));
    }   
}

fn bitflag_checkbox2(
    ui: &mut egui::Ui,
    events: &mut Vec<GuiEvent>,
    value: &str,
    flags: &mut DebugFlags,
    mask: u32,
    exclusive: u32,
) {
    let mut check_value = flags.0 & mask > 0;
    if ui.checkbox(&mut check_value, value).changed() {
        if check_value {
            flags.0 &= !exclusive;
            flags.0 |= mask;
        } else {
            flags.0 &= !mask
        }
        events.push(GuiEvent::SetDebugFlags(*flags));
    }   
}
