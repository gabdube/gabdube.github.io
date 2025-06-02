#![cfg(not(feature="gui"))]

pub struct Gui {
}

impl Gui {

    pub fn init(&self, _init: &crate::GameClientInit, _assets: &crate::data::Assets) -> Result<(), crate::Error> {
        Ok(())
    }

    pub fn update_time(&self, _delta: f32) {
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {
        
    }

    pub fn update(&self) -> bool {
        false
    }

    pub fn load_font(&mut self, _assets: &crate::data::Assets) -> Result<(), crate::Error>  {
        Ok(())
    }

    pub fn load_style(&mut self) {
    }
}

impl crate::store::StoreLoad for Gui {
    fn store(&mut self, _writer: &mut crate::store::StoreWriter) {
        
    }

    fn load(_reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        Ok(Gui::default())
    }
}

impl Default for Gui {

    fn default() -> Self {
        Gui {
        }
    }

}
