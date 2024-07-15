#![allow(dead_code)]

use super::*;
use delegate::delegate;

use core::ffi::c_void;
use core::ptr::NonNull;
use libc;
use nix::sys::mman::{MapFlags, ProtFlags};
use std::boxed::Box;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::num::NonZeroUsize;
use std::os::unix::fs::OpenOptionsExt;
use std::string::String;
use std::sync::{Arc, RwLock};

struct MmapFile {
    file: File,
    addr: NonNull<c_void>,
    size: usize,
}

unsafe impl Sync for MmapFile {}
unsafe impl Send for MmapFile {}

impl MmapFile {
    pub fn new(path: String, size: usize) -> Result<Self, Box<dyn Error>> {
        Self::new_with_flag(path, size, 0)
    }

    pub fn new_with_flag(path: String, size: usize, flag: i32) -> Result<Self, Box<dyn Error>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(flag)
            .open(path)?;
        unsafe {
            let addr = nix::sys::mman::mmap(
                None,
                NonZeroUsize::new(size).unwrap(),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                &file,
                0 as libc::off_t,
            )?;

            Ok(MmapFile {
                file: file,
                addr: addr,
                size: size,
            })
        }
    }

    pub fn addr(&self) -> usize {
        self.addr.as_ptr() as usize
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn file(&mut self) -> &File {
        &mut self.file
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, Box<dyn Error>> {
        let size = self.file.write(data)?;
        Ok(size)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Box<dyn Error>> {
        let size = self.file.read(buf)?;
        Ok(size)
    }
}

impl Drop for MmapFile {
    fn drop(&mut self) {
        unsafe {
            nix::sys::mman::munmap(self.addr, self.size as libc::size_t).unwrap();
        }
    }
}

pub struct MmapRegion {
    mfile: Arc<RwLock<MmapFile>>,
    addr: usize,
    size: usize,
}

impl MmapRegion {
    pub fn new(path: String, size: usize) -> Result<Self, Box<dyn Error>> {
        Self::new_with_flag(path, size, 0)
    }

    pub fn new_with_flag(path: String, size: usize, flag: i32) -> Result<Self, Box<dyn Error>> {
        let mfile = MmapFile::new_with_flag(path, size, flag)?;
        let addr = mfile.addr();
        let size = mfile.size();
        Ok(Self {
            mfile: Arc::new(RwLock::new(mfile)),
            addr: addr,
            size: size,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, Box<dyn Error>> {
        self.mfile.write().unwrap().write(data)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Box<dyn Error>> {
        self.mfile.write().unwrap().read(buf)
    }
}

impl MemRegion for MmapRegion {
    fn subclone(&self, offset: usize, size: usize) -> Self {
        debug_assert!(offset < self.size);
        let new_addr = self.addr + offset;
        let new_size = self.size - offset;
        debug_assert!(size <= new_size);
        let new_size = if size == 0 { new_size } else { size };
        MmapRegion {
            mfile: self.mfile.clone(),
            addr: new_addr,
            size: new_size,
        }
    }

    fn addr(&self) -> usize {
        self.addr
    }

    fn size(&self) -> usize {
        self.size
    }

    fn phys_addr(&self) -> usize {
        self.addr()
    }
}

impl Clone for MmapRegion {
    fn clone(&self) -> Self {
        self.subclone(0, 0)
    }
}

pub struct MmapAccessor<U> {
    accessor: MemAccessor<MmapRegion, U>,
}

impl<U> From<MmapAccessor<U>> for MemAccessor<MmapRegion, U> {
    fn from(from: MmapAccessor<U>) -> MemAccessor<MmapRegion, U> {
        from.accessor
    }
}

impl<U> MmapAccessor<U> {
    pub fn new(path: String, size: usize) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            accessor: MemAccessor::<MmapRegion, U>::new(MmapRegion::new(path, size)?),
        })
    }

    pub fn subclone_<NewU>(&self, offset: usize, size: usize) -> MmapAccessor<NewU> {
        MmapAccessor::<NewU> {
            accessor: MemAccessor::<MmapRegion, NewU>::new(
                self.accessor.region().subclone(offset, size),
            ),
        }
    }

    pub fn subclone8(&self, offset: usize, size: usize) -> MmapAccessor<u8> {
        self.subclone_::<u8>(offset, size)
    }

    pub fn subclone16(&self, offset: usize, size: usize) -> MmapAccessor<u16> {
        self.subclone_::<u16>(offset, size)
    }

    pub fn subclone32(&self, offset: usize, size: usize) -> MmapAccessor<u32> {
        self.subclone_::<u32>(offset, size)
    }

    pub fn subclone64(&self, offset: usize, size: usize) -> MmapAccessor<u64> {
        self.subclone_::<u64>(offset, size)
    }
}

impl<U> Clone for MmapAccessor<U> {
    fn clone(&self) -> Self {
        self.subclone(0, 0)
    }
}

impl<U> MemAccessBase for MmapAccessor<U> {
    fn reg_size() -> usize {
        core::mem::size_of::<U>()
    }

    fn subclone(&self, offset: usize, size: usize) -> Self {
        self.subclone_::<U>(offset, size)
    }

    delegate! {
        to self.accessor {
            unsafe fn copy_to_<V>(&self, src_adr: usize, dst_ptr: *mut V, count: usize);
            unsafe fn copy_from_<V>(&self, src_ptr: *const V, dst_adr: usize, count: usize);

            unsafe fn write_mem_<V>(&self, offset: usize, data: V);
            unsafe fn read_mem_<V>(&self, offset: usize) -> V;
            unsafe fn write_reg_<V>(&self, reg: usize, data: V);
            unsafe fn read_reg_<V>(&self, reg: usize) -> V;
        }
    }
}

impl<U> MemAccess for MmapAccessor<U> {
    delegate! {
        to self.accessor {
            fn addr(&self) -> usize;
            fn size(&self) -> usize;
            fn phys_addr(&self) -> usize;

            unsafe fn copy_to_usize(&self, src_adr: usize, dst_ptr: *mut usize, count: usize);
            unsafe fn copy_to_u8   (&self, src_adr: usize, dst_ptr: *mut u8   , count: usize);
            unsafe fn copy_to_u16  (&self, src_adr: usize, dst_ptr: *mut u16  , count: usize);
            unsafe fn copy_to_u32  (&self, src_adr: usize, dst_ptr: *mut u32  , count: usize);
            unsafe fn copy_to_u64  (&self, src_adr: usize, dst_ptr: *mut u64  , count: usize);
            unsafe fn copy_to_isize(&self, src_adr: usize, dst_ptr: *mut isize, count: usize);
            unsafe fn copy_to_i8   (&self, src_adr: usize, dst_ptr: *mut i8   , count: usize);
            unsafe fn copy_to_i16  (&self, src_adr: usize, dst_ptr: *mut i16  , count: usize);
            unsafe fn copy_to_i32  (&self, src_adr: usize, dst_ptr: *mut i32  , count: usize);
            unsafe fn copy_to_i64  (&self, src_adr: usize, dst_ptr: *mut i64  , count: usize);
            unsafe fn copy_to_f32  (&self, src_adr: usize, dst_ptr: *mut f32  , count: usize);
            unsafe fn copy_to_f64  (&self, src_adr: usize, dst_ptr: *mut f64  , count: usize);

            unsafe fn copy_from_usize(&self, src_ptr: *const usize, dst_adr: usize, count: usize);
            unsafe fn copy_from_u8   (&self, src_ptr: *const u8   , dst_adr: usize, count: usize);
            unsafe fn copy_from_u16  (&self, src_ptr: *const u16  , dst_adr: usize, count: usize);
            unsafe fn copy_from_u32  (&self, src_ptr: *const u32  , dst_adr: usize, count: usize);
            unsafe fn copy_from_u64  (&self, src_ptr: *const u64  , dst_adr: usize, count: usize);
            unsafe fn copy_from_isize(&self, src_ptr: *const isize, dst_adr: usize, count: usize);
            unsafe fn copy_from_i8   (&self, src_ptr: *const i8   , dst_adr: usize, count: usize);
            unsafe fn copy_from_i16  (&self, src_ptr: *const i16  , dst_adr: usize, count: usize);
            unsafe fn copy_from_i32  (&self, src_ptr: *const i32  , dst_adr: usize, count: usize);
            unsafe fn copy_from_i64  (&self, src_ptr: *const i64  , dst_adr: usize, count: usize);
            unsafe fn copy_from_f32  (&self, src_ptr: *const f32  , dst_adr: usize, count: usize);
            unsafe fn copy_from_f64  (&self, src_ptr: *const f64  , dst_adr: usize, count: usize);

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
    }
}
