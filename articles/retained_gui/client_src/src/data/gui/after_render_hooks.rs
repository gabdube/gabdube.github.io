use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use super::animations::GuiAnimationPlayState;
use super::components::GuiComponentData;
use super::{Gui, GuiComponents};

#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub(super) struct UpdateScrollView {
    pub scroll_view_index: u32,
    pub scroll_bar_vertical_index: u32,
    pub last_visible_height: f32,
    pub last_content_height: f32,
}

#[derive(Copy, Clone)]
pub(super) enum AfterRenderHook {
    UpdateAnimation(GuiAnimationPlayState),
    UpdateScrollView(UpdateScrollView)
}

impl AfterRenderHook {
    pub fn id(&self) -> u32 {
        match self {
            Self::UpdateAnimation(_) => 1,
            Self::UpdateScrollView(_) => 2,
        }
    }
}


pub(super) fn after_render(gui: &mut Gui, delta_ms: u32) -> bool {
    let mut needs_rerender = false;

    for hook in gui.after_render.iter_mut() {
        match hook {
            AfterRenderHook::UpdateAnimation(animation_state) => {
                if animation_state.animation.is_playing() {
                    needs_rerender |= animation_state.apply(delta_ms, &mut gui.components);
                }
            },
            AfterRenderHook::UpdateScrollView(view) => {
                needs_rerender |= update_scroll_view(&mut gui.components, view);
            }
        }
    }

    needs_rerender
}

fn update_scroll_view(components: &mut GuiComponents, view: &mut UpdateScrollView) -> bool {
    let scroll_view = components.copy_view(view.scroll_view_index as usize);
    if scroll_view.items_size.height == view.last_content_height && scroll_view.size.height == view.last_visible_height {
        return false;
    }

    view.last_content_height = scroll_view.items_size.height;
    view.last_visible_height = scroll_view.size.height;

    match components.get_data_mut(view.scroll_view_index as usize) {
        GuiComponentData::ScrollView(scroll_view) => scroll_view.after_render(view.last_content_height, view.last_visible_height),
        _ => warn!("scroll_view_index does not map to a scrollview component")
    }

    match components.get_data_mut(view.scroll_bar_vertical_index as usize) {
        GuiComponentData::ScrollbarVertical(bar) => bar.after_render(view.last_content_height, view.last_visible_height),
        _ => warn!("scroll_bar_vertical_index does not map to a vertical scrollbar component")
    }

    true
}

impl crate::store::StoreLoad for Vec<AfterRenderHook> {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&(self.len() as u32));
        for &item in self.iter() {
            writer.write(&item.id());

            match item {
                AfterRenderHook::UpdateAnimation(animation) => {
                    writer.write(&animation);
                },
                AfterRenderHook::UpdateScrollView(view) => {
                    writer.write(&view);
                }
            }
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let item_count = reader.try_read::<u32>()? as usize;
        let mut items = Vec::with_capacity(item_count);

        for _ in 0..item_count {
            let id = reader.try_read::<u32>()? as usize;
            match id {
                1 => {
                    let animation = reader.try_read()?;
                    items.push(AfterRenderHook::UpdateAnimation(animation));
                },
                2 => {
                    let view = reader.try_read()?;
                    items.push(AfterRenderHook::UpdateScrollView(view));
                },
                _ => {
                    return Err(assets_err!("Unknown identifier for AfterRenderHook. Data might be corrupted"));
                }
            }
        }

        Ok(items)
    }
}