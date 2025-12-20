use zerocopy::{IntoBytes, Immutable};
use std::cell::UnsafeCell;
use crate::shared::{gen_u32, align_up};

/// Locked subsection of an OutputDataBuffer
pub struct OutputDataSubBuffer<'a, T: Copy+IntoBytes+Immutable> {
    data_buffer_id: u32,
    pub data_bytes_offset: usize,
    pub data: &'a mut [T],
}

/// Generic data storage shared with the engine
pub(super) struct OutputDataBuffer {
    raw: UnsafeCell<Vec<u8>>,
    next_bytes_offset: usize,
    id: u32,
    subbuffers_count: u32,
}

impl OutputDataBuffer {
    
    pub fn with_capacity(cap: usize) -> Self {
        OutputDataBuffer {
            raw: UnsafeCell::new(vec![0; cap]),
            next_bytes_offset: 0,
            id: gen_u32(),
            subbuffers_count: 0
        }
    }
    
    pub fn clear(&mut self) {
        assert!(self.subbuffers_count == 0, "Data buffer is still borrowed somewhere");
        self.next_bytes_offset = 0;
    }

    pub fn as_ptr(&mut self) -> *const u8 {
        assert!(self.subbuffers_count == 0, "Data buffer is still borrowed somewhere");
        self.raw.get_mut().as_ptr()
    }

    // pub fn reserve<'a, 'b, T: Copy+IntoBytes+Immutable>(&'a mut self, count: usize) -> OutputDataSubBuffer<'b, T> {
    //     assert!(self.subbuffers_count == 0, "Data buffer is still borrowed somewhere");
    //     self.reserve_inner(count)
    // }

    pub fn reserve2<'a, 'b, T1, T2>(&'a mut self, count1: usize, count2: usize) -> (OutputDataSubBuffer<'b, T1>, OutputDataSubBuffer<'b, T2>)
    where
        T1: Copy+IntoBytes+Immutable,
        T2:  Copy+IntoBytes+Immutable,
    {
        assert!(self.subbuffers_count == 0, "Data buffer is still borrowed somewhere");
        let buffer1 = self.reserve_inner(count1);
        let buffer2 = self.reserve_inner(count2);
        (buffer1, buffer2)
    }

    /// Reserve enough space to hold an array of [T; count]
    /// Returned buffer needs to be released using `release`
    /// While at least one sub buffer is reserved, all other method will panic
    fn reserve_inner<'a, 'b, T: Copy+IntoBytes+Immutable>(&'a mut self, count: usize) -> OutputDataSubBuffer<'b, T> {
        if count == 0 {
            self.subbuffers_count += 1;
            return OutputDataSubBuffer {
                data_buffer_id: self.id,
                data_bytes_offset: 0,
                data: &mut [],
            };
        }
        
        // Align offset to data
        self.next_bytes_offset = align_up(self.next_bytes_offset, usize::max(align_of::<T>(), 4));
        
        // Realloc data if there is not enough space
        let total_bytes_size = count * size_of::<T>();
        if total_bytes_size > self.remaining_size() {
            self.realloc_data(total_bytes_size);
        }

        // Safety: buffer will always be large enough to hold data
        let data_bytes_offset = self.next_bytes_offset;
        let data = unsafe {
            let raw_buffer = &mut *self.raw.get();
            let slice_next = &mut raw_buffer[data_bytes_offset..data_bytes_offset+total_bytes_size];
            slice_next.align_to_mut::<T>().1
        };

        self.subbuffers_count += 1;
        self.next_bytes_offset += total_bytes_size;

        OutputDataSubBuffer {
            data_buffer_id: self.id,
            data_bytes_offset,
            data,
        }
    }

    pub fn release<T: Copy+IntoBytes+Immutable>(&mut self, other: OutputDataSubBuffer<T>) {
        assert!(other.data_buffer_id == self.id, "Sub buffer was not allocated from this buffer");
        self.subbuffers_count -= 1;
    }

    fn remaining_size(&mut self) -> usize {
        self.raw.get_mut().len() - self.next_bytes_offset
    }

    #[inline(never)]
    #[cold]
    fn realloc_data(&mut self, min_size: usize) {
        let raw = self.raw.get_mut();
        raw.reserve_exact(align_up(min_size, 0x10000));
        unsafe { raw.set_len(raw.capacity()); }
    }
}
