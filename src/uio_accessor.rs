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

fn read_file_to_string(path: String) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

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
            mmap_region: MmapRegion::new(fname, size)?,
            phys_addr: phys_addr,
        })
    }

    pub fn phys_addr(&self) -> usize {
        self.phys_addr
    }

    pub fn set_irq_enable(&mut self, enable: bool) -> Result<(), Box<dyn Error>> {
        let data: [u8; 4] = unsafe { std::mem::transmute(if enable { 1u32 } else { 0u32 }) };
        self.mmap_region.write(&data)?;
        Ok(())
    }

    pub fn wait_irq(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf: [u8; 4] = [0; 4];
        self.mmap_region.read(&mut buf)?;
        Ok(())
    }

    pub fn read_name(uio_num: usize) -> Result<String, Box<dyn Error>> {
        let fname = format!("/sys/class/uio/uio{}/name", uio_num);
        Ok(read_file_to_string(fname)?.trim().to_string())
    }

    pub fn read_size(uio_num: usize) -> Result<usize, Box<dyn Error>> {
        let fname = format!("/sys/class/uio/uio{}/maps/map0/size", uio_num);
        Ok(usize::from_str_radix(
            &read_file_to_string(fname)?.trim()[2..],
            16,
        )?)
    }

    pub fn read_phys_addr(uio_num: usize) -> Result<usize, Box<dyn Error>> {
        let fname = format!("/sys/class/uio/uio{}/maps/map0/addr", uio_num);
        Ok(usize::from_str_radix(
            &read_file_to_string(fname)?.trim()[2..],
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

    delegate! {
        to self.mmap_region {
            fn addr(&self) -> usize;
            fn size(&self) -> usize;
        }
    }
}

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

    pub fn subclone(&self, offset: usize, size: usize) -> UioAccessor<U> {
        self.subclone_::<U>(offset, size)
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
            pub fn phys_addr(&self) -> usize;
        }
        to self.mem_accessor.region_mut() {
            pub fn set_irq_enable(&mut self, enable: bool) -> Result<(), Box<dyn Error>>;
            pub fn wait_irq(&mut self) -> Result<(), Box<dyn Error>>;
        }
    }
}

impl<U> MemAccess for UioAccessor<U> {
    fn reg_size() -> usize {
        core::mem::size_of::<U>()
    }

    delegate! {
        to self.mem_accessor {
            fn addr(&self) -> usize;
            fn size(&self) -> usize;

            unsafe fn copy_to<V>(&self, src_adr: usize, dst_ptr: *mut V, count: usize);
            unsafe fn copy_from<V>(&self, src_ptr: *const V, dst_adr: usize, count: usize);

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

            unsafe fn write_memi(&self, offset: usize, data: isize);
            unsafe fn write_memi8(&self, offset: usize, data: i8);
            unsafe fn write_memi16(&self, offset: usize, data: i16);
            unsafe fn write_memi32(&self, offset: usize, data: i32);
            unsafe fn write_memi64(&self, offset: usize, data: i64);
            unsafe fn write_memf32(&self, offset: usize, data: f32);
            unsafe fn write_memf64(&self, offset: usize, data: f64);
            unsafe fn read_memi(&self, offset: usize) -> isize;
            unsafe fn read_memi8(&self, offset: usize) -> i8;
            unsafe fn read_memi16(&self, offset: usize) -> i16;
            unsafe fn read_memi32(&self, offset: usize) -> i32;
            unsafe fn read_memi64(&self, offset: usize) -> i64;
            unsafe fn read_memf32(&self, offset: usize) -> f32;
            unsafe fn read_memf64(&self, offset: usize) -> f64;
            unsafe fn write_regi(&self, reg: usize, data: isize);
            unsafe fn write_regi8(&self, reg: usize, data: i8);
            unsafe fn write_regi16(&self, reg: usize, data: i16);
            unsafe fn write_regi32(&self, reg: usize, data: i32);
            unsafe fn write_regi64(&self, reg: usize, data: i64);
            unsafe fn write_regf32(&self, reg: usize, data: f32);
            unsafe fn write_regf64(&self, reg: usize, data: f64);
            unsafe fn read_regi(&self, reg: usize) -> isize;
            unsafe fn read_regi8(&self, reg: usize) -> i8;
            unsafe fn read_regi16(&self, reg: usize) -> i16;
            unsafe fn read_regi32(&self, reg: usize) -> i32;
            unsafe fn read_regi64(&self, reg: usize) -> i64;
            unsafe fn read_regf32(&self, reg: usize) -> f32;
            unsafe fn read_regf64(&self, reg: usize) -> f64;
        }
    }
}
