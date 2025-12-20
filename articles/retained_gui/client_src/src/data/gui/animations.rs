use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::data::gui::GuiComponents;
use crate::shared::{PositionF32, pos};

/// Structure used to control animations at runtime
#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiAnimationControl {
    command_flags: u32,
}

impl GuiAnimationControl {
    pub const PLAY: u32    = 0b00001;
    pub const PAUSE: u32   = 0b00010;
    pub const RESTART: u32 = 0b00100;
}

impl GuiAnimationControl {
    #[inline(always)]
    pub fn play(&mut self) { self.command_flags |= Self::PLAY; }

    #[inline(always)]
    pub fn pause(&mut self) { self.command_flags |= Self::PAUSE; }

    #[inline(always)]
    pub fn restart(&mut self) { self.command_flags |= Self::RESTART; }

    #[inline(always)]
    pub(super) fn command_play(&self) -> bool { self.command_flags & Self::PLAY > 0 }

    #[inline(always)]
    pub(super) fn command_pause(&self) -> bool { self.command_flags & Self::PAUSE > 0 }

    #[inline(always)]
    pub(super) fn command_restart(&self) -> bool { self.command_flags & Self::RESTART > 0 }

    #[inline(always)]
    pub(super) fn has_no_updates(&self) -> bool {
        self.command_flags == 0
    }

    #[inline(always)]
    pub(super) fn clear_updates(&mut self) {
        self.command_flags = 0;
    }
}

impl Default for GuiAnimationControl {
    fn default() -> Self {
        GuiAnimationControl { command_flags: 0 }
    }
}

#[derive(Copy, Clone, Default, Immutable, IntoBytes, FromBytes)]
pub struct GuiAnimationKeyFrame {
    pub translate: PositionF32
}

#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiAnimation {
    pub flags: u32,
    pub duration_ms: u32,
    pub from: GuiAnimationKeyFrame,
    pub to: GuiAnimationKeyFrame,
}

impl GuiAnimation {
    pub const PLAYING: u32   = 0b0001;
    pub const LOOPING: u32   = 0b0010;
    pub const TRANSLATE: u32 = 0b0100;
    
    #[inline(always)]
    pub fn is_playing(&self) -> bool {
        self.flags & Self::PLAYING > 0
    }

    #[inline(always)]
    pub fn is_looping(&self) -> bool {
        self.flags & GuiAnimation::LOOPING > 0
    }

    #[inline(always)]
    pub fn set_is_playing(&mut self, playing: bool) {
        self.flags &= !Self::PLAYING;
        self.flags |= Self::PLAYING * (playing as u32);
    }
}

#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub(super) struct GuiAnimationPlayState {
    pub component_index: u32,
    pub current_runtime_ms: u32,
    pub animation: GuiAnimation,
}

impl GuiAnimationPlayState {
    pub fn new(component_index: u32, animation: GuiAnimation) -> Self {
        GuiAnimationPlayState {
            component_index,
            current_runtime_ms: 0,
            animation,
        }
    }

    fn interpolate_current_frame(&mut self, delta_ms: u32) -> GuiAnimationKeyFrame {
        let anim = self.animation;

        self.current_runtime_ms = u32::min(anim.duration_ms, self.current_runtime_ms + delta_ms);
        let p = (self.current_runtime_ms as f32) / (anim.duration_ms as f32);

        // Restart animation if looping
        let looping_and_finished = self.current_runtime_ms == anim.duration_ms && anim.is_looping();
        self.current_runtime_ms = self.current_runtime_ms * (!looping_and_finished) as u32;

        self.animation.set_is_playing(self.current_runtime_ms < anim.duration_ms);
        
        let mut key = GuiAnimationKeyFrame::default();
        key.translate = lerp_position(p, anim.from.translate, anim.to.translate, anim.flags & GuiAnimation::TRANSLATE > 0);
        key
    }

    pub fn apply(&mut self, delta_ms: u32, components: &mut GuiComponents) -> bool {
        let index = self.component_index as usize;
        let key = self.interpolate_current_frame(delta_ms);
        let layout = &mut components.get_layout_mut(index);
        layout.align_self.offset.x = key.translate.x;
        layout.align_self.offset.y = key.translate.y;

        self.animation.is_playing()
    }
}

fn lerp_position(p: f32, from: PositionF32, to: PositionF32, enabled: bool) -> PositionF32 {
    pos(
        from.x + (to.x - from.x) * p * (enabled as u32 as f32),
        from.y + (to.y - from.y) * p * (enabled as u32 as f32),
    )
}

