//! Zero-allocation memory management
//! Pre-allocated pools, no runtime heap allocation

use core::mem::MaybeUninit;

/// Fixed-size object pool - zero allocation after initialization
pub struct Pool<T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    free_indices: [u16; N],
    free_count: u16,
    used_count: u16,
}

/// Handle to a pooled object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolHandle {
    index: u16,
    generation: u16,
}

impl PoolHandle {
    pub const fn null() -> Self {
        Self { index: u16::MAX, generation: 0 }
    }
    
    pub const fn is_null(&self) -> bool {
        self.index == u16::MAX
    }
}

impl<T, const N: usize> Pool<T, N> {
    /// Create new pool (const fn - can be used in static context)
    pub const fn new() -> Self {
        let mut free_indices = [0u16; N];
        let mut i = 0;
        while i < N {
            free_indices[i] = i as u16;
            i += 1;
        }
        
        Self {
            storage: [MaybeUninit::uninit(); N],
            free_indices,
            free_count: N as u16,
            used_count: 0,
        }
    }
    
    /// Allocate object from pool
    pub fn alloc(&mut self) -> Option<PoolHandle> {
        if self.free_count == 0 {
            return None;
        }
        
        self.free_count -= 1;
        let index = self.free_indices[self.free_count as usize];
        self.used_count += 1;
        
        Some(PoolHandle { index, generation: 0 })
    }
    
    /// Free object back to pool
    pub fn free(&mut self, handle: PoolHandle) {
        if handle.is_null() || handle.index as usize >= N {
            return;
        }
        
        // Safety: index is valid and was allocated
        self.free_indices[self.free_count as usize] = handle.index;
        self.free_count += 1;
        self.used_count -= 1;
    }
    
    /// Get reference to object
    pub fn get(&self, handle: PoolHandle) -> Option<&T> {
        if handle.is_null() || handle.index as usize >= N {
            return None;
        }
        // Safety: caller ensures handle is valid
        Some(unsafe { self.storage[handle.index as usize].assume_init_ref() })
    }
    
    /// Get mutable reference to object
    pub fn get_mut(&mut self, handle: PoolHandle) -> Option<&mut T> {
        if handle.is_null() || handle.index as usize >= N {
            return None;
        }
        Some(unsafe { self.storage[handle.index as usize].assume_init_mut() })
    }
    
    /// Get uninitialized slot for manual initialization
    pub fn get_uninit_mut(&mut self, handle: PoolHandle) -> Option<&mut MaybeUninit<T>> {
        if handle.is_null() || handle.index as usize >= N {
            return None;
        }
        Some(&mut self.storage[handle.index as usize])
    }
    
    pub const fn capacity(&self) -> usize { N }
    pub const fn used(&self) -> u16 { self.used_count }
    pub const fn available(&self) -> u16 { self.free_count }
}

/// Linear arena allocator - bump allocation, reset all at once
pub struct Arena<const N: usize> {
    buffer: [u8; N],
    offset: usize,
}

impl<const N: usize> Arena<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            offset: 0,
        }
    }
    
    /// Allocate bytes from arena
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<&mut [u8]> {
        let aligned_offset = (self.offset + align - 1) & !(align - 1);
        if aligned_offset + size > N {
            return None;
        }
        let start = aligned_offset;
        self.offset = aligned_offset + size;
        Some(&mut self.buffer[start..self.offset])
    }
    
    /// Reset arena (free all allocations at once)
    pub fn reset(&mut self) {
        self.offset = 0;
    }
    
    pub const fn capacity(&self) -> usize { N }
    pub const fn used(&self) -> usize { self.offset }
}

/// Ring buffer for streaming data (vertex buffers, etc.)
pub struct RingBuffer<const N: usize> {
    buffer: [u8; N],
    write_cursor: usize,
    read_cursor: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            write_cursor: 0,
            read_cursor: 0,
        }
    }
    
    /// Write data to ring buffer
    pub fn write(&mut self, data: &[u8]) -> bool {
        if data.len() > N {
            return false;
        }
        for (i, &byte) in data.iter().enumerate() {
            self.buffer[(self.write_cursor + i) % N] = byte;
        }
        self.write_cursor = (self.write_cursor + data.len()) % N;
        true
    }
    
    /// Read data from ring buffer
    pub fn read(&mut self, len: usize) -> Option<&[u8]> {
        if len > N {
            return None;
        }
        let start = self.read_cursor;
        self.read_cursor = (self.read_cursor + len) % N;
        Some(&self.buffer[start..start + len])
    }
}
