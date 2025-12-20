use crate::data::assets::{Assets, AtlasData};
use crate::data::gui::{
    Gui, GuiState, FlexboxItemsLayout, GuiBuilder, GuiButtonStyles, GuiButtonStyle,
    GuiListViewItemStyles, GuiListViewItemStyle, GuiWindowStyle, GuiImageStyle,
    GuiAnimation, GuiAnimationControl, GuiAnimationKeyFrame, GuiInternalEvent,
};
use crate::error::Error;
use crate::shared::{ColorRGBA8, rgba8, pos, size};

const DEFAULT_TEXT_SCALE: f32 = 36.0;
const TEXT_LARGE: f32 = 60.0;
const TEXT_COLOR_LABEL: ColorRGBA8 = rgba8(40, 30, 20, 255);

#[derive(Copy, Clone, Default)]
pub enum GuiCurrentDemo {
    #[default]
    Basic,
    HelloWorld,
    Listview,
    Window,
    Animations,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum GuiEvent {
    ToggleFerris = 1,
    NextDemo = 2,
    SayHello = 3,
    ClearHello = 4,
    AnimalSelected = 5,
    PlayAnimation = 6,
    PauseAnimation = 7,
    RestartAnimation = 8,
    Last
}

#[derive(Copy, Clone)]
pub struct RunningGuiState {
    pub animals_values: &'static [&'static str],

    pub ferris_type: u32,
    pub ferris: GuiState<GuiImageStyle>,

    pub hello_text_input: GuiState<String>,
    pub hello_text_display: GuiState<String>,
    pub hello_dialog_toggle: GuiState<bool>,

    pub selected_item_text: GuiState<String>,
    pub selected_item: GuiState<usize>,

    pub drop_animation: GuiState<GuiAnimationControl>,

    pub current_demo: GuiCurrentDemo,
}

fn default_button_style(atlas: &AtlasData) -> GuiButtonStyles {
    const TEXT_COLOR_LABEL: ColorRGBA8 = rgba8(40, 30, 20, 255);
    let texture = atlas.texture;
    GuiButtonStyles {
        default: GuiButtonStyle { texture, sprite: atlas.button_default, text_color: TEXT_COLOR_LABEL, text_offset: pos(0.0, -3.0) },
        hovered: GuiButtonStyle { texture, sprite: atlas.button_hovered, text_color: TEXT_COLOR_LABEL, text_offset: pos(0.0, -3.0) },
        pressed: GuiButtonStyle { texture, sprite: atlas.button_pressed, text_color: TEXT_COLOR_LABEL, text_offset: pos(0.0, 0.0) },
    }
}

const fn default_item_view_style() -> GuiListViewItemStyles {
    const TEXT_COLOR: ColorRGBA8 = rgba8(40, 30, 20, 255);
    const BORDER_COLOR: ColorRGBA8 = rgba8(22, 28, 50, 255);
    GuiListViewItemStyles {
        default: GuiListViewItemStyle { background_color: rgba8(200, 130, 120, 255), border_color: BORDER_COLOR, text_color: TEXT_COLOR },
        hovered: GuiListViewItemStyle { background_color: rgba8(233, 142, 140, 255), border_color: BORDER_COLOR, text_color: TEXT_COLOR },
        pressed: GuiListViewItemStyle { background_color: rgba8(233, 112, 110, 255), border_color: BORDER_COLOR, text_color: TEXT_COLOR },
    }
}

const fn default_window_style() -> GuiWindowStyle {
    GuiWindowStyle { 
        title_bar_height: 45.0,
        title_bar_color: rgba8(225, 85, 85, 255),
        title_text_scale: 30.0,
        title_text_color: TEXT_COLOR_LABEL,
    }
}

const fn flexbox_layout() -> FlexboxItemsLayout {
    use crate::data::gui::{FlexDirection, FlexAlignItems, FlexJustifyContent};
    FlexboxItemsLayout {
        children_offset: pos(0.0, 0.0),
        direction: FlexDirection::Column,
        align_items: FlexAlignItems::Center,
        justify_content: FlexJustifyContent::Start,
    }
}

const fn dialog_flexbox_layout() -> FlexboxItemsLayout {
    use crate::data::gui::FlexJustifyContent;
    FlexboxItemsLayout {
        justify_content: FlexJustifyContent::Center,
        ..flexbox_layout()
    }
}

const fn row_layout() -> FlexboxItemsLayout {
    use crate::data::gui::{FlexDirection, FlexAlignItems};
    FlexboxItemsLayout { 
        direction: FlexDirection::Row,
        align_items: FlexAlignItems::Start,
        ..flexbox_layout()
    }
}

const fn drop_in_animation() -> GuiAnimation {
    GuiAnimation {
        flags: GuiAnimation::PLAYING | GuiAnimation::TRANSLATE,
        duration_ms: 1500,
        from: GuiAnimationKeyFrame {
            translate: pos(0.0, -600.0),
        },
        to: GuiAnimationKeyFrame {
            translate: pos(0.0, 0.0),
        }
    }
}

fn list_view_component(
    gui: &mut GuiBuilder,
    values: &[&str],
    selected_item: GuiState<usize>,
    on_item_click: GuiEvent,
) {
    use crate::data::gui::{FlexAlignItems, FlexDirection, FlexJustifyContent};

    let layout = FlexboxItemsLayout {
        children_offset: pos(0.0, 0.0),
        align_items: FlexAlignItems::Stretch,
        direction: FlexDirection::Column,
        justify_content: FlexJustifyContent::Start,
    };

    let style = default_item_view_style();
    let text_size = 32.0;
    let item_height = 50.0;

    gui.layout_items_flex(layout);
    gui.scroll_view(|gui| {
        gui.layout_background();
        gui.solid_color_block(rgba8(233, 140, 140, 255));

        gui.layout_background();
        gui.borders(1.0, rgba8(0, 0, 0, 255));

        gui.layout_items_flex(layout);
        gui.list_view_base(on_item_click, selected_item, &style, text_size, item_height, |gui| {
            for (id, &item) in values.iter().enumerate() {
                gui.list_view_item(id, item);
            }
        });
    });
}

fn basic_demo(assets: &Assets, gui_state: &mut RunningGuiState, gui: &mut GuiBuilder) {
    let atlas = &assets.atlas;
    let button_style = default_button_style(atlas);

    gui_state.ferris_type = 0;
    gui_state.ferris = gui.image_state(atlas.texture, atlas.ferris);

    gui.layout_center();
    gui.layout_items_flex(flexbox_layout());
    gui.group(|gui| {
        gui.image_dyn(gui_state.ferris);

        gui.spacer(0.0, 10.0);

        gui.button(GuiEvent::ToggleFerris, &button_style, "Toggle", DEFAULT_TEXT_SCALE);

        gui.spacer(0.0, 10.0);

        gui.button(GuiEvent::NextDemo, &button_style, "Next", DEFAULT_TEXT_SCALE);
    });
}

fn hello_world_demo(assets: &Assets, gui_state: &mut RunningGuiState, gui: &mut GuiBuilder) {
    let atlas = &assets.atlas;
    let button_style = default_button_style(atlas);

    gui_state.hello_dialog_toggle = gui.bool_state(false);
    gui_state.hello_text_input = gui.string_state(String::new());
    gui_state.hello_text_display = gui.string_state(String::new());

    gui.layout_center();
    gui.layout_items_flex(flexbox_layout());
    gui.group(|gui| {
        gui.layout_parent_fixed_width(320.0);
        gui.text_input(gui_state.hello_text_input, DEFAULT_TEXT_SCALE, TEXT_COLOR_LABEL);

        gui.spacer(0.0, 10.0);

        gui.button(GuiEvent::SayHello, &button_style, "Say Hello", DEFAULT_TEXT_SCALE);

        gui.spacer(0.0, 10.0);

        gui.button(GuiEvent::NextDemo, &button_style, "Next", DEFAULT_TEXT_SCALE);
    });

    gui.layout_background();
    gui.toggle(gui_state.hello_dialog_toggle, |gui| {
        gui.layout_background();
        gui.solid_color_block(rgba8(0, 0, 0, 150));

        gui.layout_center_min_size(size(420.0, 250.0));
        gui.layout_items_flex(dialog_flexbox_layout());
        gui.group(|gui| {
            gui.layout_background();
            gui.solid_color_block(rgba8(233, 140, 140, 255));

            gui.layout_background();
            gui.borders(5.0, rgba8(161, 60, 60, 255));

            gui.layout_items_flex(row_layout());
            gui.group(|gui| {
                gui.spacer(15.0, 0.0);
                gui.label_dyn(gui_state.hello_text_display, TEXT_LARGE, TEXT_COLOR_LABEL);
                gui.spacer(15.0, 0.0);
            });

            gui.spacer(0.0, 30.0);

            gui.button(GuiEvent::ClearHello, &button_style, "Return", DEFAULT_TEXT_SCALE);
        });
    });
}

fn list_view_demo(assets: &Assets, gui_state: &mut RunningGuiState, gui: &mut GuiBuilder) {
    const CONTAINER_WIDTH: f32 = 300.0;

    let atlas = &assets.atlas;
    let button_style = default_button_style(atlas);

    gui_state.selected_item_text = gui.string_state("Select an item");
    gui_state.selected_item = gui.usize_state(usize::MAX);

    gui.layout_center();
    gui.layout_items_flex(flexbox_layout());
    gui.group(|gui| {

        gui.layout_parent_fixed_size(size(CONTAINER_WIDTH, 50.0));
        gui.layout_items_flex(dialog_flexbox_layout());
        gui.group(|gui| {
            gui.layout_background();
            gui.solid_color_block(rgba8(233, 140, 140, 255));

            gui.layout_background();
            gui.borders(1.0, rgba8(0, 0, 0, 255));

            gui.label_dyn(gui_state.selected_item_text, 32.0, TEXT_COLOR_LABEL);
        });

        gui.spacer(0.0, 10.0);

        gui.layout_parent_fixed_size(size(CONTAINER_WIDTH, 250.0));
        list_view_component(
            gui,
            gui_state.animals_values,
            gui_state.selected_item,
            GuiEvent::AnimalSelected,
        );

        gui.spacer(0.0, 30.0);

        gui.button(GuiEvent::NextDemo, &button_style, "Next", DEFAULT_TEXT_SCALE);
    });
}

fn window_demo(assets: &Assets, gui: &mut GuiBuilder) {
    let atlas = &assets.atlas;
    let window_style = default_window_style();
    let button_style = default_button_style(atlas);

    gui.layout_center();
    gui.layout_items_flex(flexbox_layout());
    gui.window(&window_style, "Window demo", |gui| {
        gui.layout_background();
        gui.solid_color_block(rgba8(255, 140, 140, 255));

        gui.layout_background();
        gui.borders(1.0, rgba8(0, 0, 0, 255));

        gui.spacer(0.0, 15.0);

        gui.layout_items_flex(row_layout());
        gui.group(|gui| {
            gui.spacer(15.0, 0.0);
            gui.label("A Window Component", DEFAULT_TEXT_SCALE, TEXT_COLOR_LABEL);
            gui.spacer(15.0, 0.0);
        });
    
        gui.spacer(0.0, 30.0);

        gui.button(GuiEvent::NextDemo, &button_style, "Next", DEFAULT_TEXT_SCALE);

        gui.spacer(0.0, 15.0);
    });
}

fn animation_demo(assets: &Assets, gui_state: &mut RunningGuiState, gui: &mut GuiBuilder) {
    let atlas = &assets.atlas;
    let button_style = default_button_style(atlas);

    gui_state.drop_animation = gui.animation_state(GuiAnimationControl::default());

    gui.layout_center();
    gui.layout_items_flex(flexbox_layout());
    gui.group(|gui| {
        // If the animation doesn't need to be controlled:
        // gui.animate(drop_in_animation());  
        gui.animate_dyn(gui_state.drop_animation, drop_in_animation());
        gui.layout_parent_fixed_size(size(282.0, 185.0));
        gui.image(atlas.texture, atlas.ferris);

        gui.spacer(0.0, 40.0);

        gui.button(GuiEvent::PlayAnimation, &button_style, "Play", DEFAULT_TEXT_SCALE);

        gui.spacer(0.0, 10.0);

        gui.button(GuiEvent::PauseAnimation, &button_style, "Pause", DEFAULT_TEXT_SCALE);

        gui.spacer(0.0, 10.0);

        gui.button(GuiEvent::RestartAnimation, &button_style, "Restart", DEFAULT_TEXT_SCALE);

        gui.spacer(0.0, 10.0);

        gui.button(GuiEvent::NextDemo, &button_style, "Next", DEFAULT_TEXT_SCALE);
    });
}

pub(super) fn build(gui_module: &mut Gui, assets: &Assets, state: &mut RunningGuiState) {
    gui_module.build(|gui| {
        match state.current_demo {
            GuiCurrentDemo::Basic => basic_demo(assets, state, gui),
            GuiCurrentDemo::HelloWorld => hello_world_demo(assets, state, gui),
            GuiCurrentDemo::Listview => list_view_demo(assets, state, gui),
            GuiCurrentDemo::Window => window_demo(assets, gui),
            GuiCurrentDemo::Animations => animation_demo(assets, state, gui),
        }
    });
}

//
// Other impls
//

fn animals_list() -> &'static [&'static str] {
    &["Dog", "Cat", "Parrot", "Chinchilla", "Cow", "Lizard", "Crocodile", "Aligator", "Fish", "Monkey", "Snake", "SNAAAAAAAKE"]
}

impl crate::store::StoreLoad for RunningGuiState {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.ferris_type);
        writer.write(&(self.current_demo as u32));
        
        self.ferris.store(writer);
        self.hello_text_input.store(writer);
        self.hello_text_display.store(writer);
        self.hello_dialog_toggle.store(writer);
        self.selected_item_text.store(writer);
        self.selected_item.store(writer);
        self.drop_animation.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, Error> {
        let ferris_type = reader.try_read()?;
        let current_demo = GuiCurrentDemo::try_from(reader.try_read::<u32>()?)?;
        
        let state = RunningGuiState {
            animals_values: animals_list(),

            current_demo,
            ferris_type,
            ferris: GuiState::load(reader)?,
            hello_text_input: GuiState::load(reader)?,
            hello_text_display: GuiState::load(reader)?,
            hello_dialog_toggle: GuiState::load(reader)?,
            selected_item_text: GuiState::load(reader)?,
            selected_item: GuiState::load(reader)?,
            drop_animation: GuiState::load(reader)?,
        };

        Ok(state)
    }

}

impl Default for RunningGuiState {
    fn default() -> Self {
        RunningGuiState {
            animals_values: animals_list(),

            ferris_type: 0,
            ferris: GuiState::default(),

            hello_text_input: GuiState::default(),
            hello_text_display: GuiState::default(),
            hello_dialog_toggle: GuiState::default(),

            selected_item_text: GuiState::default(),
            selected_item: GuiState::default(),

            drop_animation: GuiState::default(),

            current_demo: GuiCurrentDemo::default(),
        }
    }
}

impl TryFrom<u32> for GuiCurrentDemo {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GuiCurrentDemo::Basic),
            1 => Ok(GuiCurrentDemo::HelloWorld),
            2 => Ok(GuiCurrentDemo::Listview),
            3 => Ok(GuiCurrentDemo::Window),
            4 => Ok(GuiCurrentDemo::Animations),
            _ => Err(assets_err!("Unknown identifier {value} for GuiCurrentDemo"))
        }
    }
}

impl Into<GuiInternalEvent> for GuiEvent {
    fn into(self) -> GuiInternalEvent {
        unsafe { GuiInternalEvent::new_unchecked(self as u32) }
    }
}

impl TryFrom<GuiInternalEvent> for GuiEvent {
    type Error = Error;
    fn try_from(value: GuiInternalEvent) -> Result<Self, Self::Error> {
        if value.get() < GuiEvent::Last as u32 {
            unsafe { Ok(::std::mem::transmute(value.get() as u8)) }
        } else {
            Err(assets_err!("Unknown identifier for GuiEvent: {value}"))
        }
    }
}
