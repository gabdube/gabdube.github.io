use zerocopy_derive::{FromBytes, Immutable, TryFromBytes, IntoBytes};

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct RenderFlags(pub u32);

impl RenderFlags {
    pub const UPDATE_GUI: u32 = 0x1;

    flags!(update_gui, set_update_gui, Self::UPDATE_GUI);
}


#[derive(Copy, Clone, PartialEq, Default, IntoBytes, TryFromBytes, Immutable)]
#[repr(u8)]
pub enum ButtonState {
    #[default]
    Released = 0,
    JustReleased = 1,
    Pressed = 2,
    JustPressed = 3,
}

impl ButtonState {
    pub fn flip(&mut self) {
        match self {
            Self::JustPressed => { *self = Self::Pressed; }
            Self::JustReleased => { *self = Self::Released; }
            _ => {}
        }
    }

    pub fn just_released(self) -> bool { self == Self::JustReleased }
    // pub fn released(self) -> bool { self == Self::JustReleased || self == Self::Released }
    pub fn just_pressed(self) -> bool { self == Self::JustPressed }
}

pub mod keys {
    #![allow(dead_code)]
    use ::std::num::NonZeroU8;

    macro_rules! generate_keys {
        ([$(($id:literal, $name:ident, $value:literal)),+]) => {
            $(pub const $name: Key = Key::from_const($value);)+

            fn key_from_str(value: &str) -> Option<Key> {
                match value {
                    $($id => Some($name),)+
                    _ => { 
                        // dbg!("{:?}", value);
                        None
                    }
                }
            }
        };
    }

    #[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
    pub struct Key(::std::num::NonZeroU8);

    pub const KEY_INVALID: Key = Key::from_const(255);

    generate_keys!([
        ("Digit1", DIGIT_1, 1),
        ("Digit2", DIGIT_2, 2),
        ("Digit3", DIGIT_3, 3),
        ("Digit4", DIGIT_4, 4),
        ("Digit5", DIGIT_5, 5),
        ("Digit6", DIGIT_6, 6),
        ("Digit7", DIGIT_7, 7),
        ("Digit8", DIGIT_8, 8),
        ("Digit9", DIGIT_9, 9),
        ("Digit0", DIGIT_0, 10),
        ("KeyA", A, 11),
        ("KeyB", B, 12),
        ("KeyC", C, 13),
        ("KeyD", D, 14),
        ("KeyE", E, 15),
        ("KeyF", F, 16),
        ("KeyG", G, 17),
        ("KeyH", H, 18),
        ("KeyI", I, 19),
        ("KeyJ", J, 20),
        ("KeyK", K, 21),
        ("KeyL", L, 22),
        ("KeyM", M, 23),
        ("KeyN", N, 24),
        ("KeyO", O, 25),
        ("KeyP", P, 26),
        ("KeyQ", Q, 27),
        ("KeyR", R, 28),
        ("KeyS", S, 29),
        ("KeyT", T, 30),
        ("KeyU", U, 31),
        ("KeyV", V, 32),
        ("KeyW", W, 33),
        ("KeyX", X, 34),
        ("KeyY", Y, 35),
        ("KeyZ", Z, 36),
        ("Escape", ESCAPE, 37),
        ("Backspace", BACKSPACE, 38),
        ("ArrowRight", ARROW_RIGHT, 39),
        ("ArrowLeft", ARROW_LEFT, 40),
        ("ArrowDown", ARROW_DOWN, 41),
        ("ArrowUp", ARROW_UP, 42)
    ]);

    impl Key {
        pub fn from_str(value: &str) -> Option<Self> {
            key_from_str(value)
        }

        pub const fn from_const(value: u8) -> Self {
            unsafe {
                match value {
                    0 => Key(NonZeroU8::new_unchecked(255)),
                    value => Key(NonZeroU8::new_unchecked(value)),
                }
            }
        }
    }
}

