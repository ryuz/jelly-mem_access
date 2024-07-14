#![allow(dead_code)]

use core::marker::PhantomData;
use core::ptr;

pub trait MemRegion {
    fn subclone(&self, offset: usize, size: usize) -> Self;
    fn addr(&self) -> usize;
    fn size(&self) -> usize;
    fn phys_addr(&self) -> usize;
}

pub trait MemAccessBase {
    fn reg_size() -> usize;

    fn subclone(&self, offset: usize, size: usize) -> Self;

    unsafe fn copy_to_<V>(&self, src_adr: usize, dst_ptr: *mut V, count: usize);
    unsafe fn copy_from_<V>(&self, src_ptr: *const V, dst_adr: usize, count: usize);

    unsafe fn write_mem_<V>(&self, offset: usize, data: V);
    unsafe fn read_mem_<V>(&self, offset: usize) -> V;
    unsafe fn write_reg_<V>(&self, reg: usize, data: V);
    unsafe fn read_reg_<V>(&self, reg: usize) -> V;
}

pub trait MemAccess {
    fn addr(&self) -> usize;
    fn size(&self) -> usize;
    fn phys_addr(&self) -> usize;

    unsafe fn copy_to_usize(&self, src_adr: usize, dst_ptr: *mut usize, count: usize);
    unsafe fn copy_to_u8(&self, src_adr: usize, dst_ptr: *mut u8, count: usize);
    unsafe fn copy_to_u16(&self, src_adr: usize, dst_ptr: *mut u16, count: usize);
    unsafe fn copy_to_u32(&self, src_adr: usize, dst_ptr: *mut u32, count: usize);
    unsafe fn copy_to_u64(&self, src_adr: usize, dst_ptr: *mut u64, count: usize);
    unsafe fn copy_to_isize(&self, src_adr: usize, dst_ptr: *mut isize, count: usize);
    unsafe fn copy_to_i8(&self, src_adr: usize, dst_ptr: *mut i8, count: usize);
    unsafe fn copy_to_i16(&self, src_adr: usize, dst_ptr: *mut i16, count: usize);
    unsafe fn copy_to_i32(&self, src_adr: usize, dst_ptr: *mut i32, count: usize);
    unsafe fn copy_to_i64(&self, src_adr: usize, dst_ptr: *mut i64, count: usize);
    unsafe fn copy_to_f32(&self, src_adr: usize, dst_ptr: *mut f32, count: usize);
    unsafe fn copy_to_f64(&self, src_adr: usize, dst_ptr: *mut f64, count: usize);

    unsafe fn copy_from_usize(&self, src_ptr: *const usize, dst_adr: usize, count: usize);
    unsafe fn copy_from_u8(&self, src_ptr: *const u8, dst_adr: usize, count: usize);
    unsafe fn copy_from_u16(&self, src_ptr: *const u16, dst_adr: usize, count: usize);
    unsafe fn copy_from_u32(&self, src_ptr: *const u32, dst_adr: usize, count: usize);
    unsafe fn copy_from_u64(&self, src_ptr: *const u64, dst_adr: usize, count: usize);
    unsafe fn copy_from_isize(&self, src_ptr: *const isize, dst_adr: usize, count: usize);
    unsafe fn copy_from_i8(&self, src_ptr: *const i8, dst_adr: usize, count: usize);
    unsafe fn copy_from_i16(&self, src_ptr: *const i16, dst_adr: usize, count: usize);
    unsafe fn copy_from_i32(&self, src_ptr: *const i32, dst_adr: usize, count: usize);
    unsafe fn copy_from_i64(&self, src_ptr: *const i64, dst_adr: usize, count: usize);
    unsafe fn copy_from_f32(&self, src_ptr: *const f32, dst_adr: usize, count: usize);
    unsafe fn copy_from_f64(&self, src_ptr: *const f64, dst_adr: usize, count: usize);

    unsafe fn write_mem(&self, offset: usize, data: usize);
    unsafe fn write_mem_usize(&self, offset: usize, data: usize);
    unsafe fn write_mem_u8(&self, offset: usize, data: u8);
    unsafe fn write_mem_u16(&self, offset: usize, data: u16);
    unsafe fn write_mem_u32(&self, offset: usize, data: u32);
    unsafe fn write_mem_u64(&self, offset: usize, data: u64);
    unsafe fn write_mem_isize(&self, offset: usize, data: isize);
    unsafe fn write_mem_i8(&self, offset: usize, data: i8);
    unsafe fn write_mem_i16(&self, offset: usize, data: i16);
    unsafe fn write_mem_i32(&self, offset: usize, data: i32);
    unsafe fn write_mem_i64(&self, offset: usize, data: i64);
    unsafe fn write_mem_f32(&self, offset: usize, data: f32);
    unsafe fn write_mem_f64(&self, offset: usize, data: f64);

    unsafe fn read_mem(&self, offset: usize) -> usize;
    unsafe fn read_mem_usize(&self, offset: usize) -> usize;
    unsafe fn read_mem_u8(&self, offset: usize) -> u8;
    unsafe fn read_mem_u16(&self, offset: usize) -> u16;
    unsafe fn read_mem_u32(&self, offset: usize) -> u32;
    unsafe fn read_mem_u64(&self, offset: usize) -> u64;
    unsafe fn read_mem_isize(&self, offset: usize) -> isize;
    unsafe fn read_mem_i8(&self, offset: usize) -> i8;
    unsafe fn read_mem_i16(&self, offset: usize) -> i16;
    unsafe fn read_mem_i32(&self, offset: usize) -> i32;
    unsafe fn read_mem_i64(&self, offset: usize) -> i64;
    unsafe fn read_mem_f32(&self, offset: usize) -> f32;
    unsafe fn read_mem_f64(&self, offset: usize) -> f64;

    unsafe fn write_reg(&self, reg: usize, data: usize);
    unsafe fn write_reg_usize(&self, reg: usize, data: usize);
    unsafe fn write_reg_u8(&self, reg: usize, data: u8);
    unsafe fn write_reg_u16(&self, reg: usize, data: u16);
    unsafe fn write_reg_u32(&self, reg: usize, data: u32);
    unsafe fn write_reg_u64(&self, reg: usize, data: u64);
    unsafe fn write_reg_isize(&self, reg: usize, data: isize);
    unsafe fn write_reg_i8(&self, reg: usize, data: i8);
    unsafe fn write_reg_i16(&self, reg: usize, data: i16);
    unsafe fn write_reg_i32(&self, reg: usize, data: i32);
    unsafe fn write_reg_i64(&self, reg: usize, data: i64);
    unsafe fn write_reg_f32(&self, reg: usize, data: f32);
    unsafe fn write_reg_f64(&self, reg: usize, data: f64);

    unsafe fn read_reg(&self, reg: usize) -> usize;
    unsafe fn read_reg_usize(&self, reg: usize) -> usize;
    unsafe fn read_reg_u8(&self, reg: usize) -> u8;
    unsafe fn read_reg_u16(&self, reg: usize) -> u16;
    unsafe fn read_reg_u32(&self, reg: usize) -> u32;
    unsafe fn read_reg_u64(&self, reg: usize) -> u64;
    unsafe fn read_reg_isize(&self, reg: usize) -> isize;
    unsafe fn read_reg_i8(&self, reg: usize) -> i8;
    unsafe fn read_reg_i16(&self, reg: usize) -> i16;
    unsafe fn read_reg_i32(&self, reg: usize) -> i32;
    unsafe fn read_reg_i64(&self, reg: usize) -> i64;
    unsafe fn read_reg_f32(&self, reg: usize) -> f32;
    unsafe fn read_reg_f64(&self, reg: usize) -> f64;
}

pub trait MemAccessCache {
    unsafe fn cache_flush(&self, offset: usize, size: usize);
    unsafe fn cache_flush_all(&self);
    unsafe fn cache_invalidate(&self, offset: usize, size: usize);
    unsafe fn cache_invalidate_all(&self);
}

pub trait MemAccessSync {
    unsafe fn sync_owner(&self) -> u32;
    unsafe fn sync_for_cpu(&self);
    unsafe fn sync_for_cpu_with_range(&self, sync_offset: usize, sync_size: usize, sync_direction: u32, sync_for_cpu: u32);
    unsafe fn sync_for_device(&self);
    unsafe fn sync_for_device_with_range(&self, sync_offset: usize, sync_size: usize, sync_direction: u32, sync_for_device: u32);
}


pub struct MemAccessor<T: MemRegion, U> {
    region: T,
    phantom: PhantomData<U>,
}

impl<T: MemRegion, U> MemAccessor<T, U> {
    pub const fn new(region: T) -> Self {
        MemAccessor::<T, U> {
            region: region,
            phantom: PhantomData,
        }
    }

    pub fn region(&self) -> &T {
        &self.region
    }

    pub fn region_mut(&mut self) -> &mut T {
        &mut self.region
    }

    pub fn subclone_<NewU>(&self, offset: usize, size: usize) -> MemAccessor<T, NewU> {
        MemAccessor::<T, NewU>::new(self.region.subclone(offset, size))
    }

    pub fn subclone8(&self, offset: usize, size: usize) -> MemAccessor<T, u8> {
        self.subclone_::<u8>(offset, size)
    }

    pub fn subclone16(&self, offset: usize, size: usize) -> MemAccessor<T, u16> {
        self.subclone_::<u16>(offset, size)
    }

    pub fn subclone32(&self, offset: usize, size: usize) -> MemAccessor<T, u32> {
        self.subclone_::<u32>(offset, size)
    }

    pub fn subclone64(&self, offset: usize, size: usize) -> MemAccessor<T, u64> {
        self.subclone_::<u64>(offset, size)
    }
}

impl<T: MemRegion, U> Clone for MemAccessor<T, U> {
    fn clone(&self) -> Self {
        self.subclone(0, 0)
    }
}

impl<T: MemRegion, U> MemAccessBase for MemAccessor<T, U> {
    fn reg_size() -> usize {
        core::mem::size_of::<U>()
    }

    fn subclone(&self, offset: usize, size: usize) -> Self {
        self.subclone_::<U>(offset, size)
    }

    unsafe fn copy_to_<V>(&self, src_adr: usize, dst_ptr: *mut V, count: usize) {
        assert!(src_adr + count * core::mem::size_of::<V>() <= self.size());
        let src_ptr: *const V = (self.addr() + src_adr) as *const V;
        core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, count);
    }

    unsafe fn copy_from_<V>(&self, src_ptr: *const V, dst_adr: usize, count: usize) {
        assert!(dst_adr + count * core::mem::size_of::<V>() <= self.size());
        let dst_ptr: *mut V = (self.addr() + dst_adr) as *mut V;
        core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, count);
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
}

impl<T: MemRegion, U> MemAccess for MemAccessor<T, U> {
    fn addr(&self) -> usize {
        self.region.addr()
    }

    fn size(&self) -> usize {
        self.region.size()
    }

    fn phys_addr(&self) -> usize {
        self.region.phys_addr()
    }

    unsafe fn copy_to_usize(&self, src_adr: usize, dst_ptr: *mut usize, count: usize) {
        self.copy_to_::<usize>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_u8(&self, src_adr: usize, dst_ptr: *mut u8, count: usize) {
        self.copy_to_::<u8>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_u16(&self, src_adr: usize, dst_ptr: *mut u16, count: usize) {
        self.copy_to_::<u16>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_u32(&self, src_adr: usize, dst_ptr: *mut u32, count: usize) {
        self.copy_to_::<u32>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_u64(&self, src_adr: usize, dst_ptr: *mut u64, count: usize) {
        self.copy_to_::<u64>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_isize(&self, src_adr: usize, dst_ptr: *mut isize, count: usize) {
        self.copy_to_::<isize>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_i8(&self, src_adr: usize, dst_ptr: *mut i8, count: usize) {
        self.copy_to_::<i8>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_i16(&self, src_adr: usize, dst_ptr: *mut i16, count: usize) {
        self.copy_to_::<i16>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_i32(&self, src_adr: usize, dst_ptr: *mut i32, count: usize) {
        self.copy_to_::<i32>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_i64(&self, src_adr: usize, dst_ptr: *mut i64, count: usize) {
        self.copy_to_::<i64>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_f32(&self, src_adr: usize, dst_ptr: *mut f32, count: usize) {
        self.copy_to_::<f32>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_to_f64(&self, src_adr: usize, dst_ptr: *mut f64, count: usize) {
        self.copy_to_::<f64>(src_adr, dst_ptr, count)
    }

    unsafe fn copy_from_usize(&self, src_ptr: *const usize, dst_adr: usize, count: usize) {
        self.copy_from_::<usize>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_u8(&self, src_ptr: *const u8, dst_adr: usize, count: usize) {
        self.copy_from_::<u8>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_u16(&self, src_ptr: *const u16, dst_adr: usize, count: usize) {
        self.copy_from_::<u16>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_u32(&self, src_ptr: *const u32, dst_adr: usize, count: usize) {
        self.copy_from_::<u32>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_u64(&self, src_ptr: *const u64, dst_adr: usize, count: usize) {
        self.copy_from_::<u64>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_isize(&self, src_ptr: *const isize, dst_adr: usize, count: usize) {
        self.copy_from_::<isize>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_i8(&self, src_ptr: *const i8, dst_adr: usize, count: usize) {
        self.copy_from_::<i8>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_i16(&self, src_ptr: *const i16, dst_adr: usize, count: usize) {
        self.copy_from_::<i16>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_i32(&self, src_ptr: *const i32, dst_adr: usize, count: usize) {
        self.copy_from_::<i32>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_i64(&self, src_ptr: *const i64, dst_adr: usize, count: usize) {
        self.copy_from_::<i64>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_f32(&self, src_ptr: *const f32, dst_adr: usize, count: usize) {
        self.copy_from_::<f32>(src_ptr, dst_adr, count)
    }

    unsafe fn copy_from_f64(&self, src_ptr: *const f64, dst_adr: usize, count: usize) {
        self.copy_from_::<f64>(src_ptr, dst_adr, count)
    }

    unsafe fn write_mem(&self, offset: usize, data: usize) {
        self.write_mem_::<usize>(offset, data)
    }

    unsafe fn write_mem_usize(&self, offset: usize, data: usize) {
        self.write_mem_::<usize>(offset, data)
    }

    unsafe fn write_mem_u8(&self, offset: usize, data: u8) {
        self.write_mem_::<u8>(offset, data)
    }

    unsafe fn write_mem_u16(&self, offset: usize, data: u16) {
        self.write_mem_::<u16>(offset, data)
    }

    unsafe fn write_mem_u32(&self, offset: usize, data: u32) {
        self.write_mem_::<u32>(offset, data)
    }

    unsafe fn write_mem_u64(&self, offset: usize, data: u64) {
        self.write_mem_::<u64>(offset, data)
    }

    unsafe fn write_mem_isize(&self, offset: usize, data: isize) {
        self.write_mem_::<isize>(offset, data)
    }

    unsafe fn write_mem_i8(&self, offset: usize, data: i8) {
        self.write_mem_::<i8>(offset, data)
    }

    unsafe fn write_mem_i16(&self, offset: usize, data: i16) {
        self.write_mem_::<i16>(offset, data)
    }

    unsafe fn write_mem_i32(&self, offset: usize, data: i32) {
        self.write_mem_::<i32>(offset, data)
    }

    unsafe fn write_mem_i64(&self, offset: usize, data: i64) {
        self.write_mem_::<i64>(offset, data)
    }

    unsafe fn write_mem_f32(&self, offset: usize, data: f32) {
        self.write_mem_::<f32>(offset, data)
    }

    unsafe fn write_mem_f64(&self, offset: usize, data: f64) {
        self.write_mem_::<f64>(offset, data)
    }

    unsafe fn read_mem(&self, offset: usize) -> usize {
        self.read_mem_::<usize>(offset)
    }

    unsafe fn read_mem_usize(&self, offset: usize) -> usize {
        self.read_mem_::<usize>(offset)
    }

    unsafe fn read_mem_u8(&self, offset: usize) -> u8 {
        self.read_mem_::<u8>(offset)
    }

    unsafe fn read_mem_u16(&self, offset: usize) -> u16 {
        self.read_mem_::<u16>(offset)
    }

    unsafe fn read_mem_u32(&self, offset: usize) -> u32 {
        self.read_mem_::<u32>(offset)
    }

    unsafe fn read_mem_u64(&self, offset: usize) -> u64 {
        self.read_mem_::<u64>(offset)
    }

    unsafe fn read_mem_isize(&self, offset: usize) -> isize {
        self.read_mem_::<isize>(offset)
    }

    unsafe fn read_mem_i8(&self, offset: usize) -> i8 {
        self.read_mem_::<i8>(offset)
    }

    unsafe fn read_mem_i16(&self, offset: usize) -> i16 {
        self.read_mem_::<i16>(offset)
    }

    unsafe fn read_mem_i32(&self, offset: usize) -> i32 {
        self.read_mem_::<i32>(offset)
    }

    unsafe fn read_mem_i64(&self, offset: usize) -> i64 {
        self.read_mem_::<i64>(offset)
    }

    unsafe fn read_mem_f32(&self, offset: usize) -> f32 {
        self.read_mem_::<f32>(offset)
    }

    unsafe fn read_mem_f64(&self, offset: usize) -> f64 {
        self.read_mem_::<f64>(offset)
    }

    unsafe fn write_reg(&self, reg: usize, data: usize) {
        self.write_reg_::<usize>(reg, data)
    }

    unsafe fn write_reg_usize(&self, reg: usize, data: usize) {
        self.write_reg_::<usize>(reg, data)
    }

    unsafe fn write_reg_u8(&self, reg: usize, data: u8) {
        self.write_reg_::<u8>(reg, data)
    }

    unsafe fn write_reg_u16(&self, reg: usize, data: u16) {
        self.write_reg_::<u16>(reg, data)
    }

    unsafe fn write_reg_u32(&self, reg: usize, data: u32) {
        self.write_reg_::<u32>(reg, data)
    }

    unsafe fn write_reg_u64(&self, reg: usize, data: u64) {
        self.write_reg_::<u64>(reg, data)
    }

    unsafe fn write_reg_isize(&self, reg: usize, data: isize) {
        self.write_reg_::<isize>(reg, data)
    }

    unsafe fn write_reg_i8(&self, reg: usize, data: i8) {
        self.write_reg_::<i8>(reg, data)
    }

    unsafe fn write_reg_i16(&self, reg: usize, data: i16) {
        self.write_reg_::<i16>(reg, data)
    }

    unsafe fn write_reg_i32(&self, reg: usize, data: i32) {
        self.write_reg_::<i32>(reg, data)
    }

    unsafe fn write_reg_i64(&self, reg: usize, data: i64) {
        self.write_reg_::<i64>(reg, data)
    }

    unsafe fn write_reg_f32(&self, reg: usize, data: f32) {
        self.write_reg_::<f32>(reg, data)
    }

    unsafe fn write_reg_f64(&self, reg: usize, data: f64) {
        self.write_reg_::<f64>(reg, data)
    }

    unsafe fn read_reg(&self, reg: usize) -> usize {
        self.read_reg_::<usize>(reg)
    }

    unsafe fn read_reg_usize(&self, reg: usize) -> usize {
        self.read_reg_::<usize>(reg)
    }

    unsafe fn read_reg_u8(&self, reg: usize) -> u8 {
        self.read_reg_::<u8>(reg)
    }

    unsafe fn read_reg_u16(&self, reg: usize) -> u16 {
        self.read_reg_::<u16>(reg)
    }

    unsafe fn read_reg_u32(&self, reg: usize) -> u32 {
        self.read_reg_::<u32>(reg)
    }

    unsafe fn read_reg_u64(&self, reg: usize) -> u64 {
        self.read_reg_::<u64>(reg)
    }

    unsafe fn read_reg_isize(&self, reg: usize) -> isize {
        self.read_reg_::<isize>(reg)
    }

    unsafe fn read_reg_i8(&self, reg: usize) -> i8 {
        self.read_reg_::<i8>(reg)
    }

    unsafe fn read_reg_i16(&self, reg: usize) -> i16 {
        self.read_reg_::<i16>(reg)
    }

    unsafe fn read_reg_i32(&self, reg: usize) -> i32 {
        self.read_reg_::<i32>(reg)
    }

    unsafe fn read_reg_i64(&self, reg: usize) -> i64 {
        self.read_reg_::<i64>(reg)
    }

    unsafe fn read_reg_f32(&self, reg: usize) -> f32 {
        self.read_reg_::<f32>(reg)
    }

    unsafe fn read_reg_f64(&self, reg: usize) -> f64 {
        self.read_reg_::<f64>(reg)
    }
}
