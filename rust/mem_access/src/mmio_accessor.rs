
#![allow(dead_code)]

use super::*;

// for Memory mapped IO
pub struct MmioRegion {
    addr: usize,
    size: usize,
}

impl MmioRegion {
    pub const fn new(addr: usize, size: usize) -> Self {
        MmioRegion { addr:addr, size:size }
    }
}


impl MemRegion for MmioRegion {
    fn clone(&self, offset: usize, size: usize) -> Self {
        debug_assert!(offset < self.size);
        let new_addr = self.addr + offset;
        let new_size = self.size - offset;
        debug_assert!(size <= new_size);
        let new_size = if size == 0 { new_size } else { size };
        MmioRegion { addr:new_addr, size:new_size }
    }

    fn addr(&self) -> usize {
        self.addr
    }

    fn size(&self) -> usize {
        self.size
    }
}


pub const fn mmio_accesor_new<U>(addr: usize, size: usize) -> MemAccesor::<MmioRegion, U>
{
    MemAccesor::<MmioRegion, U>::new(MmioRegion::new(addr, size))
}


