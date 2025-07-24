#![allow(dead_code)]

use super::*;
use delegate::delegate;
use std::boxed::Box;
use std::error::Error;
use std::format;
use std::fs::File;
use std::io::Read;
use std::string::String;
use std::string::ToString;
use thiserror::Error;

#[derive(Debug, Error)]
enum UioAccessorError {
    #[error("UioError: {0}")]
    UioError(String),
}

fn read_file_to_string(path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

#[derive(Debug)]
pub struct UioRegion {
    mmap_region: MmapRegion,
    phys_addr: usize,
}

impl UioRegion {
    pub fn new(uio_num: usize) -> Result<Self, Box<dyn Error>> {
        let phys_addr = Self::read_phys_addr(uio_num)?;
        let size = Self::read_size(uio_num)?;
        let fname = format!("/dev/uio{}", uio_num);
        Ok(UioRegion {
            mmap_region: MmapRegion::new(&fname, 0, size)?,
            phys_addr: phys_addr,
        })
    }

    pub fn set_irq_enable(&mut self, enable: bool) -> Result<(), Box<dyn Error>> {
        let data: [u8; 4] = unsafe { std::mem::transmute(if enable { 1u32 } else { 0u32 }) };
        self.mmap_region.write(&data)?;
        Ok(())
    }

    pub fn wait_irq(&mut self) -> Result<u32, Box<dyn Error>> {
        let mut buf: [u8; 4] = [0; 4];
        self.mmap_region.read(&mut buf)?;
        let count = u32::from_ne_bytes(buf);
        Ok(count)
    }

    pub fn peek_irq(&self, timeout_ms: i32) -> Result<bool, Box<dyn Error>> {
        self.mmap_region.poll(timeout_ms)
    }

    pub fn poll_irq(&mut self, timeout_ms: i32) -> Result<Option<u32>, Box<dyn Error>> {
        if self.peek_irq(timeout_ms)? {
            let irq_count = self.wait_irq()?;
            return Ok(Some(irq_count));
        }
        Ok(None) // Timeout
    }

    pub fn read_name(uio_num: usize) -> Result<String, Box<dyn Error>> {
        let fname = format!("/sys/class/uio/uio{}/name", uio_num);
        Ok(read_file_to_string(&fname)?.trim().to_string())
    }

    pub fn read_size(uio_num: usize) -> Result<usize, Box<dyn Error>> {
        let fname = format!("/sys/class/uio/uio{}/maps/map0/size", uio_num);
        Ok(usize::from_str_radix(
            &read_file_to_string(&fname)?.trim()[2..],
            16,
        )?)
    }

    pub fn read_phys_addr(uio_num: usize) -> Result<usize, Box<dyn Error>> {
        let fname = format!("/sys/class/uio/uio{}/maps/map0/addr", uio_num);
        Ok(usize::from_str_radix(
            &read_file_to_string(&fname)?.trim()[2..],
            16,
        )?)
    }
}

impl MemRegion for UioRegion {
    fn subclone(&self, offset: usize, size: usize) -> Self {
        UioRegion {
            mmap_region: self.mmap_region.subclone(offset, size),
            phys_addr: self.phys_addr + offset,
        }
    }

    fn phys_addr(&self) -> usize {
        self.phys_addr
    }

    delegate! {
        to self.mmap_region {
            fn addr(&self) -> usize;
            fn size(&self) -> usize;
        }
    }
}

#[derive(Debug)]
pub struct UioAccessor<U> {
    mem_accessor: MemAccessor<UioRegion, U>,
}

impl<U> From<UioAccessor<U>> for MemAccessor<UioRegion, U> {
    fn from(from: UioAccessor<U>) -> MemAccessor<UioRegion, U> {
        from.mem_accessor
    }
}

impl<U> UioAccessor<U> {
    pub fn new(uio_num: usize) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            mem_accessor: MemAccessor::<UioRegion, U>::new(UioRegion::new(uio_num)?),
        })
    }

    pub fn new_with_name(name: &str) -> Result<Self, Box<dyn Error>> {
        for path in std::fs::read_dir("/sys/class/uio/")? {
            let uio_num: usize = path
                .unwrap()
                .path()
                .display()
                .to_string()
                .replacen("/sys/class/uio/uio", "", 1)
                .parse()
                .unwrap();
            let dev_name = UioRegion::read_name(uio_num)?;
            if dev_name == name {
                return Self::new(uio_num);
            }
        }
        Err(Box::new(UioAccessorError::UioError(
            "device not found".to_string(),
        )))
    }

    pub fn subclone_<NewU>(&self, offset: usize, size: usize) -> UioAccessor<NewU> {
        UioAccessor::<NewU> {
            mem_accessor: MemAccessor::<UioRegion, NewU>::new(
                self.mem_accessor.region().subclone(offset, size),
            ),
        }
    }

    pub fn subclone8(&self, offset: usize, size: usize) -> UioAccessor<u8> {
        self.subclone_::<u8>(offset, size)
    }

    pub fn subclone16(&self, offset: usize, size: usize) -> UioAccessor<u16> {
        self.subclone_::<u16>(offset, size)
    }

    pub fn subclone32(&self, offset: usize, size: usize) -> UioAccessor<u32> {
        self.subclone_::<u32>(offset, size)
    }

    pub fn subclone64(&self, offset: usize, size: usize) -> UioAccessor<u64> {
        self.subclone_::<u64>(offset, size)
    }

    delegate! {
        to self.mem_accessor.region() {
            pub fn addr(&self) -> usize;
            pub fn size(&self) -> usize;
            pub fn peek_irq(&self, timeout_ms: i32) -> Result<bool, Box<dyn Error>>;
        }
        to self.mem_accessor.region_mut() {
            pub fn set_irq_enable(&mut self, enable: bool) -> Result<(), Box<dyn Error>>;
            pub fn wait_irq(&mut self) -> Result<u32, Box<dyn Error>>;
            pub fn poll_irq(&mut self, timeout_ms: i32) -> Result<Option<u32>, Box<dyn Error>>;
        }
    }
}

impl<U> MemAccessBase for UioAccessor<U> {
    fn reg_size() -> usize {
        core::mem::size_of::<U>()
    }

    fn subclone(&self, offset: usize, size: usize) -> UioAccessor<U> {
        self.subclone_::<U>(offset, size)
    }

    delegate! {
        to self.mem_accessor {
            unsafe fn copy_to_<V>(&self, src_adr: usize, dst_ptr: *mut V, count: usize);
            unsafe fn copy_from_<V>(&self, src_ptr: *const V, dst_adr: usize, count: usize);

            unsafe fn write_mem_<V>(&self, offset: usize, data: V);
            unsafe fn read_mem_<V>(&self, offset: usize) -> V;
            unsafe fn write_reg_<V>(&self, reg: usize, data: V);
            unsafe fn read_reg_<V>(&self, reg: usize) -> V;
        }
    }
}

impl<U> MemAccess for UioAccessor<U> {
    delegate! {
        to self.mem_accessor {
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
