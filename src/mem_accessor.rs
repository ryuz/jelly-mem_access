#![allow(dead_code)]

use core::marker::PhantomData;
use core::ptr;

pub trait MemRegion {
    fn clone(&self, offset: usize, size: usize) -> Self;
    fn addr(&self) -> usize;
    fn size(&self) -> usize;
}

pub trait MemAccess {
    fn reg_size() -> usize;

    unsafe fn write_mem_<V>(&self, offset: usize, data: V);
    unsafe fn read_mem_<V>(&self, offset: usize) -> V;
    unsafe fn write_reg_<V>(&self, reg: usize, data: V);
    unsafe fn read_reg_<V>(&self, reg: usize) -> V;

    unsafe fn write_mem(&self, offset: usize, data: usize);
    unsafe fn write_mem8(&self, offset: usize, data: u8);
    unsafe fn write_mem16(&self, offset: usize, data: u16);
    unsafe fn write_mem32(&self, offset: usize, data: u32);
    unsafe fn write_mem64(&self, offset: usize, data: u64);
    unsafe fn read_mem(&self, offset: usize) -> usize;
    unsafe fn read_mem8(&self, offset: usize) -> u8;
    unsafe fn read_mem16(&self, offset: usize) -> u16;
    unsafe fn read_mem32(&self, offset: usize) -> u32;
    unsafe fn read_mem64(&self, offset: usize) -> u64;

    unsafe fn write_reg(&self, reg: usize, data: usize);
    unsafe fn write_reg8(&self, reg: usize, data: u8);
    unsafe fn write_reg16(&self, reg: usize, data: u16);
    unsafe fn write_reg32(&self, reg: usize, data: u32);
    unsafe fn write_reg64(&self, reg: usize, data: u64);
    unsafe fn read_reg(&self, reg: usize) -> usize;
    unsafe fn read_reg8(&self, reg: usize) -> u8;
    unsafe fn read_reg16(&self, reg: usize) -> u16;
    unsafe fn read_reg32(&self, reg: usize) -> u32;
    unsafe fn read_reg64(&self, reg: usize) -> u64;
}

pub struct MemAccesor<T: MemRegion, U> {
    region: T,
    phantom: PhantomData<U>,
}

impl<T: MemRegion, U> MemAccesor<T, U> {
    pub const fn new(region: T) -> Self {
        MemAccesor::<T, U> {
            region: region,
            phantom: PhantomData,
        }
    }

    pub fn region(&self) -> &T {
        &self.region
    }

    pub fn clone_<NewU>(&self, offset: usize, size: usize) -> MemAccesor<T, NewU> {
        MemAccesor::<T, NewU>::new(self.region.clone(offset, size))
    }

    pub fn clone(&self, offset: usize, size: usize) -> MemAccesor<T, U> {
        self.clone_::<U>(offset, size)
    }

    pub fn clone8(&self, offset: usize, size: usize) -> MemAccesor<T, u8> {
        self.clone_::<u8>(offset, size)
    }

    pub fn clone16(&self, offset: usize, size: usize) -> MemAccesor<T, u16> {
        self.clone_::<u16>(offset, size)
    }

    pub fn clone32(&self, offset: usize, size: usize) -> MemAccesor<T, u32> {
        self.clone_::<u32>(offset, size)
    }

    pub fn clone64(&self, offset: usize, size: usize) -> MemAccesor<T, u64> {
        self.clone_::<u64>(offset, size)
    }
}

impl<T: MemRegion, U> MemAccess for MemAccesor<T, U> {
    fn reg_size() -> usize {
        core::mem::size_of::<U>()
    }

    unsafe fn write_mem_<V>(&self, offset: usize, data: V) {
        debug_assert!(offset + core::mem::size_of::<V>() <= self.region.size());
        let addr = self.region.addr() + offset;
        ptr::write_volatile(addr as *mut V, data);
    }

    unsafe fn read_mem_<V>(&self, offset: usize) -> V {
        debug_assert!(offset + core::mem::size_of::<V>() <= self.region.size());
        let addr = self.region.addr() + offset;
        ptr::read_volatile(addr as *mut V)
    }

    unsafe fn write_reg_<V>(&self, reg: usize, data: V) {
        self.write_mem_::<V>(reg * Self::reg_size(), data)
    }

    unsafe fn read_reg_<V>(&self, reg: usize) -> V {
        self.read_mem_::<V>(reg * Self::reg_size())
    }


    unsafe fn write_mem(&self, offset: usize, data: usize) {
        self.write_mem_::<usize>(offset, data)
    }

    unsafe fn write_mem8(&self, offset: usize, data: u8) {
        self.write_mem_::<u8>(offset, data)
    }

    unsafe fn write_mem16(&self, offset: usize, data: u16) {
        self.write_mem_::<u16>(offset, data)
    }

    unsafe fn write_mem32(&self, offset: usize, data: u32) {
        self.write_mem_::<u32>(offset, data)
    }

    unsafe fn write_mem64(&self, offset: usize, data: u64) {
        self.write_mem_::<u64>(offset, data)
    }

    unsafe fn read_mem(&self, offset: usize) -> usize {
        self.read_mem_::<usize>(offset)
    }

    unsafe fn read_mem8(&self, offset: usize) -> u8 {
        self.read_mem_::<u8>(offset)
    }

    unsafe fn read_mem16(&self, offset: usize) -> u16 {
        self.read_mem_::<u16>(offset)
    }

    unsafe fn read_mem32(&self, offset: usize) -> u32 {
        self.read_mem_::<u32>(offset)
    }

    unsafe fn read_mem64(&self, offset: usize) -> u64 {
        self.read_mem_::<u64>(offset)
    }

    unsafe fn write_reg(&self, reg: usize, data: usize) {
        self.write_reg_::<usize>(reg, data)
    }

    unsafe fn write_reg8(&self, reg: usize, data: u8) {
        self.write_reg_::<u8>(reg, data)
    }

    unsafe fn write_reg16(&self, reg: usize, data: u16) {
        self.write_reg_::<u16>(reg, data)
    }

    unsafe fn write_reg32(&self, reg: usize, data: u32) {
        self.write_reg_::<u32>(reg, data)
    }

    unsafe fn write_reg64(&self, reg: usize, data: u64) {
        self.write_reg_::<u64>(reg, data)
    }

    unsafe fn read_reg(&self, reg: usize) -> usize {
        self.read_reg_::<usize>(reg)
    }

    unsafe fn read_reg8(&self, reg: usize) -> u8 {
        self.read_reg_::<u8>(reg)
    }

    unsafe fn read_reg16(&self, reg: usize) -> u16 {
        self.read_reg_::<u16>(reg)
    }

    unsafe fn read_reg32(&self, reg: usize) -> u32 {
        self.read_reg_::<u32>(reg)
    }

    unsafe fn read_reg64(&self, reg: usize) -> u64 {
        self.read_reg_::<u64>(reg)
    }
}
