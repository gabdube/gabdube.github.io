//! Serializer and Deserializer for the application data
use zerocopy::{transmute, FromBytes, Immutable, IntoBytes, TryFromBytes};
use crate::error::Error;

pub const U32_SIZE: usize = size_of::<u32>();
pub const MIN_ALIGN: usize = U32_SIZE;

pub trait StoreLoad: Sized {
    fn store(&mut self, writer: &mut StoreWriter);
    fn load(reader: &mut StoreReader) -> Result<Self, Error>;
}

pub struct StoreWriter {
    pub data: Vec<u8>,
    pub data_offset: usize,
}

impl StoreWriter {

    pub fn new() -> Self {
        StoreWriter {
            data: vec![0u8; 1024*1024],    // 1mb should be more than enough
            data_offset: 0
        }
    }

    pub fn data(self) -> Box<[u8]> {
        self.data[0..self.data_offset].to_vec().into_boxed_slice()
    }

    pub fn write<T: IntoBytes+Immutable>(&mut self, value: &T) {
        assert!(align_of::<T>() == MIN_ALIGN, "Data alignment must be 4 bytes");

        let size = size_of::<T>();
        if self.must_realloc(size) {
            self.realloc(size);
        }

        value.write_to_prefix(self.remaining_bytes()).unwrap();
        self.data_offset += size;
    }

    pub fn write_option<T: IntoBytes+Immutable>(&mut self, op_value: &Option<T>) {
        let total_size = size_of::<T>() + U32_SIZE;
        if self.must_realloc(total_size) {
            self.realloc(total_size);
        }

        self.write_u32(op_value.is_some() as u32);
        if let Some(value) = op_value.as_ref() {
            value.write_to_prefix(self.remaining_bytes()).unwrap();
            self.data_offset += size_of::<T>();
        }
    }

    pub fn write_array<T: IntoBytes+Immutable>(&mut self, values: &[T]) {
        assert!(align_of::<T>() <= MIN_ALIGN, "Data alignment must up to 4 bytes");

        let values_count = values.len();
        let values_size = size_of::<T>() * values_count;
        let values_size_padded = crate::shared::align_up(values_size, MIN_ALIGN);
        let total_size = U32_SIZE + U32_SIZE + values_size_padded;
        if self.must_realloc(total_size) {
            self.realloc(total_size);
        }

        self.write(&[values_count as u32, values_size_padded as u32]);
        if values_count == 0 {
            return;
        }

        values.write_to_prefix(self.remaining_bytes()).unwrap();
        self.data_offset += values_size_padded;
    }

    pub fn write_str(&mut self, value: &str) {
        // Strings must be padded to 4 bytes
        let length = value.len();
        let padded_length = crate::shared::align_up(length, MIN_ALIGN);
        let total_length = U32_SIZE + U32_SIZE + padded_length;
        if self.must_realloc(total_length) {
            self.realloc(total_length);
        }

        self.write_u32(length as u32);
        self.write_u32(padded_length as u32);
        value.write_to_prefix(self.remaining_bytes()).unwrap();

        self.data_offset += padded_length;
    }

    pub fn write_string_hashmap<T: IntoBytes+Immutable>(&mut self, values: &fnv::FnvHashMap<String, T>) {
        assert!(align_of::<T>() == MIN_ALIGN, "Data alignment must be 4 bytes");

        let values_count = values.len() as u32;
        self.write(&values_count);

        if values_count == 0 {
            return;
        }
        
        for (key, value) in values.iter() {
            self.write_str(key);
            self.write(value);
        }
    }

    pub fn write_string_array_hashmap(&mut self, values: &fnv::FnvHashMap<String, Vec<u8>>) {
        let values_count = values.len() as u32;
        self.write(&values_count);

        if values_count == 0 {
            return;
        }
        
        for (key, value) in values.iter() {
            self.write_str(key);
            self.write_array(value);
        }
    }

    pub fn write_bool(&mut self, value: bool) {
        if self.must_realloc(U32_SIZE) {
            self.realloc(U32_SIZE);
        }

        self.write_u32(value as u32)
    }

    pub fn write_entity_option(&mut self, value: Option<hecs::Entity>) { 
        let size = U32_SIZE + U32_SIZE;
        if self.must_realloc(size) {
            self.realloc(size);
        }

        let raw_values: [u32; 2] = value
            .map(|v| v.to_bits() )
            .map(|v| transmute!(v) )
            .unwrap_or([0, 0]);

        raw_values.write_to_prefix(self.remaining_bytes()).unwrap();
        self.data_offset += size;
    }

    fn must_realloc(&self, size: usize) -> bool {
        self.data[self.data_offset..].len() < size
    }

    fn remaining_bytes(&mut self) -> &mut [u8] {
        &mut self.data[self.data_offset..]
    }

    fn write_u32(&mut self, value: u32) {
        // The caller function must ensure there is enough remaining bytes
        value.write_to_prefix(self.remaining_bytes()).unwrap();
        self.data_offset += U32_SIZE;
    }  

    #[inline(never)]
    #[cold]
    fn realloc(&mut self, min_size: usize) {
        self.data.reserve_exact(crate::shared::align_up(min_size, 0x10000));
        unsafe { self.data.set_len(self.data.capacity()); }
    }

}

pub struct StoreReader<'a> {
    pub data: &'a [u8],
    pub data_offset: usize,
}

impl<'a> StoreReader<'a> {

    pub fn new(data: &'a [u8]) -> Result<Self, Error> {
        let reader = StoreReader {
            data,
            data_offset: 0,
        };

        Ok(reader)
    }

    pub fn try_read<T: TryFromBytes+Immutable>(&mut self) -> Result<T, Error> {
        let (value, _) = TryFromBytes::try_read_from_prefix(& self.data[self.data_offset..])
            .map_err(|_| save_err!("Failed to read data") )?;

        self.data_offset += size_of::<T>();

        Ok(value)
    }

    pub fn try_read_option<T: TryFromBytes+Immutable>(&mut self) -> Result<Option<T>, Error> {
        let is_some = self.read_u32();
        if is_some > 0 {
            self.try_read().map(Option::Some)
        } else {
            Ok(None)
        }
    }

    pub fn try_read_bool(&mut self) -> Result<bool, Error> {
        self.try_read::<u32>().map(|v| v == 1 )
    }

    pub fn try_read_entity_option(&mut self) -> Result<Option<hecs::Entity>, Error> {
        let raw: [u32; 2] = self.try_read()?;
        if raw[0] == 0 && raw[1] == 0 {
            Ok(None)
        } else {
            match hecs::Entity::from_bits(transmute!(raw)) {
                Some(entity) => Ok(Some(entity)),
                None => Ok(None),
            }
        }
    }

    pub fn read_array<'b, T: Copy+FromBytes+Immutable>(&mut self) -> &'b [T] {
        unsafe { self.read_array_transmute() }
    }

    // Read array of values that may not be safe to cast `FromBytes`
    pub unsafe fn read_array_transmute<'b, T: Copy>(&mut self) -> &'b [T] {
        let count = self.read_u32() as usize;
        let values_size_padded = self.read_u32() as usize;
        if count == 0 {
            return &[];
        }

        let total_size = count * size_of::<T>();
        if total_size > self.remaining_size() {
            panic!("Malformed data. Reading array would go outside buffer");
        }

        let start_offset = self.data_offset;
        self.data_offset += values_size_padded;

        unsafe { ::std::slice::from_raw_parts(self.data.as_ptr().add(start_offset) as *const T, count) }
    }

    pub fn read_str(&mut self) -> &str {
        let length = self.read_u32();
        let length_padded = self.read_u32();
        let str = unsafe {
            let str_ptr = self.data.as_ptr().add(self.data_offset) as *const u8;
            let str_bytes = ::std::slice::from_raw_parts(str_ptr, length as usize);
            ::std::str::from_utf8(str_bytes).unwrap_or("UTF8 DECODING ERROR")
        };

        self.data_offset += length_padded as usize;

        str
    }

    pub fn read_string_hashmap<T: FromBytes+Immutable>(&mut self) -> fnv::FnvHashMap<String, T> {
        let mut out = fnv::FnvHashMap::default();
        let count = self.read_u32() as usize;
        if count == 0 {
            return out;
        }

        for _ in 0..count {
            let key = self.read_str().to_string();
            let value = self.try_read().unwrap();
            out.insert(key, value);
        }

        out
    }

    pub fn read_string_array_hashmap(&mut self) -> fnv::FnvHashMap<String, Vec<u8>> {
        let mut out = fnv::FnvHashMap::default();
        let count = self.read_u32() as usize;
        if count == 0 {
            return out;
        }

        for _ in 0..count {
            let key = self.read_str().to_string();
            let value: Vec<u8> = self.read_array().to_vec();
            out.insert(key, value);
        }

        out
    }

    fn read_u32(&mut self) -> u32 {
        let offset = self.data_offset;
        let value = u32::read_from_bytes(&self.data[offset..offset+U32_SIZE]).unwrap();
        self.data_offset += U32_SIZE;
        value
    }

    fn remaining_size(&self) -> usize {
        self.data[self.data_offset..].len()
    }
}
