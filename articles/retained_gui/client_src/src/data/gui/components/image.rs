use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::data::assets::Texture;
use crate::data::gui::components::GuiComponentView;
use crate::data::gui::{GuiStateStore, GuiOutputSprite};
use crate::data::sprites::StaticSprite;

#[derive(Default, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiImageStyle {
    pub texture: Texture,
    pub sprite: StaticSprite,
}


#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiComponentImage {
    pub texture: Texture,
    pub sprite: StaticSprite
}

impl GuiComponentImage {
    pub fn sync_state_data(&mut self, data: &GuiStateStore) {
        match data {
            GuiStateStore::Image(image) => {
                self.texture = image.texture;
                self.sprite = image.sprite;
            },
            _ => {
                warn!("Unknown state data sent to image: {:?}", data.type_name())
            }
        }
    }

    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&self, view: &GuiComponentView, callback: &mut F) {
        use crate::data::gui::generate_sprites::generate_image;
        generate_image(self.sprite.texcoord, self.texture.id, view, callback);
    }
}

//
// Builder code
// 

use crate::data::gui::components::GuiComponentData;
use crate::data::gui::{GuiBuilder, GuiState};

impl<'a> GuiBuilder<'a> {

    pub fn image(&mut self, texture: Texture, sprite: StaticSprite) {
        let image = GuiComponentImage { texture, sprite };
        self.push(GuiComponentData::Image(image));
    }

    pub fn image_dyn(&mut self, state: GuiState<GuiImageStyle>) {
        let image_style = match self.inner.state_alloc.get(state) {
            Some(style) => style,
            None => { warn!("Gui state object invalid."); return; }
        };
        let image = GuiComponentImage { 
            texture: image_style.texture,
            sprite: image_style.sprite,
        };
        let component_index = self.push(GuiComponentData::Image(image));
        self.inner.state_alloc.insert_component_listener(state, component_index);
    }

}
