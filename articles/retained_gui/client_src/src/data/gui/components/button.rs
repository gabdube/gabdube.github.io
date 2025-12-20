use zerocopy_derive::{Immutable, IntoBytes, FromBytes};

use crate::data::assets::{Texture, TextMetrics};
use crate::data::gui::components::GuiComponentView;
use crate::data::gui::{GuiOutputSprite, GuiInternalEvent, GuiOutputEvents};
use crate::data::sprites::StaticSprite;
use crate::error::Error; 
use crate::shared::{size, SizeF32, PositionF32, ColorRGBA8};


#[derive(Default, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiButtonStyle {
    pub texture: Texture,
    pub sprite: StaticSprite,
    pub text_color: ColorRGBA8,
    pub text_offset: PositionF32,
}

#[derive(Default, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiButtonStyles {
    pub default: GuiButtonStyle,
    pub hovered: GuiButtonStyle,
    pub pressed: GuiButtonStyle,
}


#[derive(Copy, Clone, PartialEq)]
pub enum GuiComponentButtonState {
    Default,
    Hovered,
    Pressed,
}

impl GuiComponentButtonState {
    pub fn from_u32(value: u32) -> Result<Self, Error> {
        match value {
            0 => Ok(GuiComponentButtonState::Default),
            1 => Ok(GuiComponentButtonState::Hovered),
            2 => Ok(GuiComponentButtonState::Pressed),
            _ => Err(assets_err!("Unkown identifier {value} for GuiComponentButtonState."))
        }
    }
}

pub struct GuiComponentButton {
    pub styles: Box<GuiButtonStyles>,
    pub text: Box<TextMetrics>,
    pub size: SizeF32,
    pub on_click: GuiInternalEvent,
    pub state: GuiComponentButtonState
}

impl GuiComponentButton {
    pub fn new(on_click: GuiInternalEvent, styles: GuiButtonStyles, text: TextMetrics,) -> Self {
        let mut button = GuiComponentButton {
            styles: Box::new(styles),
            text: Box::new(text),
            size: size(0.0, 0.0),
            on_click,
            state: GuiComponentButtonState::Default,
        };

        button.precompute_params();

        button
    }

    pub fn minimum_size(&self) -> SizeF32 {
        self.size
    }

    fn precompute_params(&mut self) {
        let sprite_texcoord = match self.state {
            GuiComponentButtonState::Default => self.styles.default.sprite.texcoord,
            GuiComponentButtonState::Hovered => self.styles.hovered.sprite.texcoord,
            GuiComponentButtonState::Pressed => self.styles.pressed.sprite.texcoord
        };
        self.size = self.text.size.max(sprite_texcoord.size());
    }

    pub fn update_mouse_state(
        &mut self,
        events: &mut GuiOutputEvents,
        is_pressed: bool,
        is_hovered: bool,
        is_clicked: bool
    ) -> bool {
        let old_state = self.state;
        self.state = match (is_pressed, is_hovered) {
            (true, _) => GuiComponentButtonState::Pressed,
            (false, true) => GuiComponentButtonState::Hovered,
            (false, false) => GuiComponentButtonState::Default
        };

        if is_clicked {
            events.push(Some(self.on_click));
        }

        old_state != self.state
    }

    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&self, view: &GuiComponentView, callback: &mut F) {
        use crate::data::gui::generate_sprites::{generate_image, generate_text};

        let style = match self.state {
            GuiComponentButtonState::Default => self.styles.default,
            GuiComponentButtonState::Hovered => self.styles.hovered,
            GuiComponentButtonState::Pressed => self.styles.pressed,
        };

        generate_image(style.sprite.texcoord, style.texture.id, view, callback);

        // Center the text on the button
        let mut text_view = *view;
        let [offset_x, offset_y] = style.text_offset.splat();
        text_view.position.x += offset_x + (view.size.width - self.text.size.width) / 2.0;
        text_view.position.y += offset_y + (view.size.height - self.text.size.height) / 2.0;

        generate_text(&self.text, &text_view, style.text_color, callback);
    }
}


//
// Builder code
// 

use crate::data::gui::components::GuiComponentData;
use crate::data::gui::GuiBuilder;

impl<'a> GuiBuilder<'a> {

    pub fn button<S: AsRef<str>, E: Into<GuiInternalEvent>>(
        &mut self,
        on_click: E,
        style: &GuiButtonStyles,
        text: S,
        text_scale: f32,
    ) {
        let text = self.inner.assets.default_font.compute_text_metrics(text.as_ref(), text_scale);
        let button = GuiComponentButton::new(on_click.into(), *style, text);
        self.push(GuiComponentData::Button(button));
    }

}
