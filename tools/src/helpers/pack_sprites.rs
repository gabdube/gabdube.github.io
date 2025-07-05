use crate::shared::{RectU32, SizeU32, rect_u32};

pub struct PackSprite {
    pub index: u32,
    pub size: SizeU32,
    pub rect: RectU32,
}

struct PackStateRef<'a> {
    sprites: &'a mut [PackSprite],
    processed: &'a mut [bool],
    area: RectU32
}

impl<'a> PackStateRef<'a> {
    fn store_next_sprite(&mut self) -> Option<SizeU32> {
        let index = self.processed.iter_mut().enumerate()
            .position(|(index, processed)| {
                if *processed == false {
                    let size = self.sprites[index].size;
                    self.area.fits(size.width, size.height)
                } else {
                    false
                }
            })?;

        let pack = &mut self.sprites[index];
        let size = pack.size;
        pack.rect = RectU32 { 
            left: self.area.left,
            top: self.area.top,
            right: self.area.left + size.width,
            bottom: self.area.top + size.height
        };

        self.processed[index] = true;

        Some(size)
    }

    fn has_remaining_items(&self) -> bool {
        self.processed.iter().any(|processed| *processed )
    }
}

pub struct SpritePackingHelper {
    sprites: Vec<PackSprite>,
    processed: Vec<bool>,
    size: SizeU32,
}

impl SpritePackingHelper {
    pub fn new(max_width: u32, mut sprites: Vec<PackSprite>) -> Self {
        use std::cmp::Ordering;

        fn sort_sprites(sprite1: &PackSprite, sprite2: &PackSprite) -> Ordering {
            if sprite2.size.height < sprite1.size.height {
                return Ordering::Less;
            } else if sprite2.size.height > sprite1.size.height {
                return Ordering::Greater;
            } else {
                return sprite2.size.width.cmp(&sprite1.size.width);
            }
        }

        let sprites_count = sprites.len();
        sprites.sort_unstable_by(sort_sprites);

        let mut pack_state = SpritePackingHelper {
            sprites,
            processed: vec![false; sprites_count],
            size: SizeU32::default()
        };

        pack_state.compute(max_width);

        pack_state
    }

    pub fn sprites(&self) -> impl Iterator<Item=&PackSprite> {
        self.sprites.iter()
    }

    pub fn size(&self) -> SizeU32 {
        self.size
    }

    fn compute(&mut self, max_width: u32) {
        let mut top = 0;

        loop {
            let size = match self.processed.iter().enumerate().find(|(_, processed)| **processed == false ) {
                Some((index, _)) => self.sprites[index].size,
                None => { break; }
            };
    
            let pack_state = PackStateRef {
                sprites: &mut self.sprites,
                processed: &mut self.processed,
                area: rect_u32(0, top, max_width, top+size.height)
            };

            if !Self::pack_row(pack_state) {
                break;
            }
    
            top += size.height;
        }

        self.size.width = max_width;
        self.size.height = top;
    }

    fn pack_row(mut state: PackStateRef) -> bool {
        loop {
            let size = match state.store_next_sprite() {
                Some(value) => value,
                None => { return state.has_remaining_items(); }
            };
    
            if size.height <= state.area.height() {
                let state = PackStateRef {
                    sprites: state.sprites,
                    processed: state.processed,
                    area: RectU32 {
                        left: state.area.left,
                        top: state.area.top + size.height,
                        right: state.area.left + size.width,
                        bottom: state.area.bottom,
                    }
                };
                if !Self::pack_row(state) {
                    return false;
                }
            }
    
            state.area.left += size.width;
        }
    }
}

