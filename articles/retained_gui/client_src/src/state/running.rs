mod build_gui;
use build_gui::{RunningGuiState, GuiEvent, GuiCurrentDemo};

use crate::data::assets::AtlasData;
use crate::data::gui::{Gui, GuiImageStyle};
use crate::error::Error;
use crate::GameClient;

#[derive(Default)]
pub struct RunningState {
    pub gui_state: RunningGuiState,
}

pub fn rebuild_running_gui(client: &mut GameClient) {
    let running_state = client.state.running();
    build_gui::build(&mut client.data.gui, &client.data.assets, &mut running_state.gui_state);

    // This sets a flag to tell the engine the gui has changed and needs to be re-rendered.
    client.data.update_gui();
}

fn toggle_ferris(state: &mut RunningGuiState, gui: &mut Gui, atlas: &AtlasData) {
    let image = match state.ferris_type {
        0 => {
            state.ferris_type = 1;
            GuiImageStyle { texture: atlas.texture, sprite: atlas.ferris_happy }
        },
        _ => {
            state.ferris_type = 0;
            GuiImageStyle { texture: atlas.texture, sprite: atlas.ferris }
        }
    };

    gui.set_state(state.ferris, image);
}

fn next_state(state: &mut RunningGuiState) {
    state.current_demo = match state.current_demo {
        GuiCurrentDemo::Basic => GuiCurrentDemo::HelloWorld,
        GuiCurrentDemo::HelloWorld => GuiCurrentDemo::Listview,
        GuiCurrentDemo::Listview => GuiCurrentDemo::Window,
        GuiCurrentDemo::Window => GuiCurrentDemo::Animations,
        GuiCurrentDemo::Animations => GuiCurrentDemo::Basic,
    };
}

pub fn update(client: &mut GameClient) {
    let gui_state = &mut client.state.running().gui_state;
    let gui = &mut client.data.gui;
    let mut update_gui = false;
    let mut rebuild_gui = false;

    while let Some(Ok(event)) = gui.read_next_event() {
        match event {
            GuiEvent::ToggleFerris => {
                let atlas = &client.data.assets.atlas;
                toggle_ferris(gui_state, gui, atlas);
                update_gui = true;
            },
            GuiEvent::NextDemo => {
                next_state(gui_state);
                rebuild_gui = true;
            },
            GuiEvent::SayHello => {
                let mut text_display = None;
                if let Some(input_text) = gui.get_state(gui_state.hello_text_input) {
                    if input_text.len() > 0 {
                        text_display = Some(format!("Hello {}!", input_text))
                    }
                };

                gui.set_state(gui_state.hello_text_display, text_display.unwrap_or_else(|| "Hello World!".to_owned() ));
                gui.set_state(gui_state.hello_dialog_toggle, true);
                update_gui = true;
            },
            GuiEvent::ClearHello => {
                gui.set_state(gui_state.hello_dialog_toggle, false);
                update_gui = true;
            },
            GuiEvent::AnimalSelected => {
                let selected_index = gui.get_state(gui_state.selected_item).copied().unwrap_or(usize::MAX);
                if let Some(&text) = gui_state.animals_values.get(selected_index) {
                    gui.set_state(gui_state.selected_item_text, String::from(text));
                    update_gui = true;
                }
            },
            GuiEvent::PlayAnimation => {
                gui.mutate_state(gui_state.drop_animation, |animation| animation.play() );
            },
            GuiEvent::PauseAnimation => {
                gui.mutate_state(gui_state.drop_animation, |animation| animation.pause() );
            },
            GuiEvent::RestartAnimation => {
                gui.mutate_state(gui_state.drop_animation, |animation| animation.restart() );
            },
            GuiEvent::Last => {
                warn!("GuiEvent::Last cannot be triggered");
            }
        }
    }

    if rebuild_gui {
        rebuild_running_gui(client)
    } else if update_gui {
        client.data.update_gui();
    }
}


//
// Other impl
//

impl crate::store::StoreLoad for RunningState {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        self.gui_state.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, Error> {
        let state = RunningState { 
            gui_state: RunningGuiState::load(reader)?
        };
        
        Ok(state)
    }
}


