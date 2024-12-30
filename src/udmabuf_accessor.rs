#![allow(dead_code)]

use super::*;
use delegate::delegate;
use std::boxed::Box;
use std::error::Error;
use std::format;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::string::String;

const O_SYNC: i32 = 0x101000;

// -----------------------------
//  Static API
// -----------------------------

fn read_file_to_string(path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(&path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn write_file_from_string(path: &str, text: &str) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).open(&path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

pub fn read_phys_addr(device_name: &str, module_name: &str) -> Result<usize, Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/phys_addr", module_name, device_name);
    Ok(usize::from_str_radix(
        &read_file_to_string(&fname)?.trim()[2..],
        16,
    )?)
}

pub fn read_size(device_name: &str, module_name: &str) -> Result<usize, Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/size", module_name, device_name);
    Ok(read_file_to_string(&fname)?.trim().parse()?)
}

pub fn read_sync_mode(device_name: &str, module_name: &str) -> Result<u32, Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_mode", module_name, device_name);
    Ok(read_file_to_string(&fname)?.trim().parse()?)
}

pub fn write_sync_mode(
    device_name: &str,
    module_name: &str,
    sync_mode: u32,
) -> Result<(), Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_mode", module_name, device_name);
    let text = format!("{}", sync_mode);
    write_file_from_string(&fname, text.as_str())
}

pub fn read_sync_offset(device_name: &str, module_name: &str) -> Result<usize, Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_offset", module_name, device_name);
    Ok(read_file_to_string(&fname)?.trim().parse()?)
}

pub fn write_sync_offset(
    device_name: &str,
    module_name: &str,
    sync_offset: usize,
) -> Result<(), Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_offset", module_name, device_name);
    let text = format!("{}", sync_offset);
    write_file_from_string(&fname, text.as_str())
}

pub fn read_sync_size(device_name: &str, module_name: &str) -> Result<usize, Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_size", module_name, device_name);
    Ok(read_file_to_string(&fname)?.trim().parse()?)
}

pub fn write_sync_size(
    device_name: &str,
    module_name: &str,
    sync_size: usize,
) -> Result<(), Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_size", module_name, device_name);
    let text = format!("{}", sync_size);
    write_file_from_string(&fname, text.as_str())
}

pub fn read_sync_direction(device_name: &str, module_name: &str) -> Result<u32, Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_direction", module_name, device_name);
    Ok(read_file_to_string(&fname)?.trim().parse()?)
}

pub fn write_sync_directione(
    device_name: &str,
    module_name: &str,
    sync_size: usize,
) -> Result<(), Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_direction", module_name, device_name);
    let text = format!("{}", sync_size);
    write_file_from_string(&fname, text.as_str())
}

pub fn read_dma_coherent(device_name: &str, module_name: &str) -> Result<u32, Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/dma_coherent", module_name, device_name);
    Ok(read_file_to_string(&fname)?.trim().parse()?)
}

pub fn read_sync_owner(device_name: &str, module_name: &str) -> Result<u32, Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_owner", module_name, device_name);
    Ok(read_file_to_string(&fname)?.trim().parse()?)
}

pub fn write_sync_for_cpu(device_name: &str, module_name: &str) -> Result<(), Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_for_cpu", module_name, device_name);
    write_file_from_string(&fname, "1")
}

pub fn write_sync_for_cpu_with_range(
    device_name: &str,
    module_name: &str,
    sync_offset: usize,
    sync_size: usize,
    sync_direction: u32,
    sync_for_cpu: u32,
) -> Result<(), Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_for_cpu", module_name, device_name);
    let text = format!(
        "0x{:08X}{:08X}",
        (sync_offset & 0xFFFFFFFF) as u32,
        (sync_size & 0xFFFFFFF0) as u32 | (sync_direction << 2) | sync_for_cpu
    );
    write_file_from_string(&fname, text.as_str())
}

pub fn write_sync_for_device(device_name: &str, module_name: &str) -> Result<(), Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_for_device", module_name, device_name);
    write_file_from_string(&fname, "1")
}

pub fn write_sync_for_device_with_range(
    device_name: &str,
    module_name: &str,
    sync_offset: usize,
    sync_size: usize,
    sync_direction: u32,
    sync_for_device: u32,
) -> Result<(), Box<dyn Error>> {
    let fname = format!("/sys/class/{}/{}/sync_for_device", module_name, device_name);
    let text = format!(
        "0x{:08X}{:08X}",
        (sync_offset & 0xFFFFFFFF) as u32,
        (sync_size & 0xFFFFFFF0) as u32 | (sync_direction << 2) | sync_for_device
    );
    write_file_from_string(&fname, text.as_str())
}

// -----------------------------
//  Udmabuf
// -----------------------------

struct UdmabufRegion {
    mmap_region: MmapRegion,
    phys_addr: usize,
    module_name: String,
    device_name: String,
}

impl UdmabufRegion {
    pub fn new(device_name: &str, cache_enable: bool) -> Result<Self, Box<dyn Error>> {
        Self::new_with_module_name(device_name, "u-dma-buf", cache_enable)
    }

    pub fn new_with_module_name(
        device_name: &str,
        module_name: &str,
        cache_enable: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let phys_addr = read_phys_addr(device_name, module_name)?;
        let size = read_size(device_name, module_name)?;

        let fname = format!("/dev/{}", device_name);
        let mmap_region =
            MmapRegion::new_with_flag(&fname, 0, size, if cache_enable { 0 } else { O_SYNC })?;

        Ok(Self {
            mmap_region: mmap_region,
            phys_addr: phys_addr,
            module_name: String::from(module_name),
            device_name: String::from(device_name),
        })
    }

    pub fn new_with_number(udmabuf_num: usize, cache_enable: bool) -> Result<Self, Box<dyn Error>> {
        let device_name = format!("udmabuf{}", udmabuf_num);
        Self::new(&device_name, cache_enable)
    }

    pub fn read_phys_addr(&self) -> Result<usize, Box<dyn Error>> {
        read_phys_addr(&self.device_name, &self.module_name)
    }

    pub fn read_phys_size(&self) -> Result<usize, Box<dyn Error>> {
        read_size(&self.device_name, &self.module_name)
    }

    pub fn read_sync_mode(&self) -> Result<u32, Box<dyn Error>> {
        read_sync_mode(&self.device_name, &self.module_name)
    }

    pub fn write_sync_mode(&self, sync_mode: u32) -> Result<(), Box<dyn Error>> {
        write_sync_mode(&self.device_name, &self.module_name, sync_mode)
    }

    pub fn read_sync_offset(&self) -> Result<usize, Box<dyn Error>> {
        read_sync_offset(&self.device_name, &self.module_name)
    }

    pub fn write_sync_offset(&self, sync_offset: usize) -> Result<(), Box<dyn Error>> {
        write_sync_offset(&self.device_name, &self.module_name, sync_offset)
    }

    pub fn read_sync_size(&self) -> Result<usize, Box<dyn Error>> {
        read_sync_size(&self.device_name, &self.module_name)
    }

    pub fn write_sync_size(&self, sync_size: usize) -> Result<(), Box<dyn Error>> {
        write_sync_size(&self.device_name, &self.module_name, sync_size)
    }

    pub fn read_sync_direction(&self) -> Result<u32, Box<dyn Error>> {
        read_sync_direction(&self.device_name, &self.module_name)
    }

    pub fn write_sync_direction(&self, sync_size: usize) -> Result<(), Box<dyn Error>> {
        write_sync_directione(&self.device_name, &self.module_name, sync_size)
    }

    pub fn read_dma_coherent(&self) -> Result<u32, Box<dyn Error>> {
        read_dma_coherent(&self.device_name, &self.module_name)
    }

    pub fn read_sync_owner(&self) -> Result<u32, Box<dyn Error>> {
        read_sync_owner(&self.device_name, &self.module_name)
    }

    pub fn write_sync_for_cpu(&self) -> Result<(), Box<dyn Error>> {
        write_sync_for_cpu(&self.device_name, &self.module_name)
    }

    pub fn write_sync_for_cpu_with_range(
        &self,
        sync_offset: usize,
        sync_size: usize,
        sync_direction: u32,
        sync_for_cpu: u32,
    ) -> Result<(), Box<dyn Error>> {
        write_sync_for_cpu_with_range(
            &self.device_name,
            &self.module_name,
            sync_offset,
            sync_size,
            sync_direction,
            sync_for_cpu,
        )
    }

    pub fn write_sync_for_device(&self) -> Result<(), Box<dyn Error>> {
        write_sync_for_device(&self.device_name, &self.module_name)
    }

    pub fn write_sync_for_device_with_range(
        &self,
        sync_offset: usize,
        sync_size: usize,
        sync_direction: u32,
        sync_for_device: u32,
    ) -> Result<(), Box<dyn Error>> {
        write_sync_for_device_with_range(
            &self.device_name,
            &self.module_name,
            sync_offset,
            sync_size,
            sync_direction,
            sync_for_device,
        )
    }
}

impl MemRegion for UdmabufRegion {
    fn subclone(&self, offset: usize, size: usize) -> Self {
        UdmabufRegion {
            mmap_region: self.mmap_region.subclone(offset, size),
            phys_addr: self.phys_addr + offset,
            module_name: self.module_name.clone(),
            device_name: self.device_name.clone(),
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

impl Clone for UdmabufRegion {
    fn clone(&self) -> Self {
        self.subclone(0, 0)
    }
}

pub struct UdmabufAccessor<U> {
    mem_accessor: MemAccessor<UdmabufRegion, U>,
}

impl<U> From<UdmabufAccessor<U>> for MemAccessor<UdmabufRegion, U> {
    fn from(from: UdmabufAccessor<U>) -> MemAccessor<UdmabufRegion, U> {
        from.mem_accessor
    }
}

impl<U> UdmabufAccessor<U> {
    pub fn new(device_name: &str, cache_enable: bool) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            mem_accessor: MemAccessor::<UdmabufRegion, U>::new(UdmabufRegion::new(
                device_name,
                cache_enable,
            )?),
        })
    }

    pub fn new_with_module_name(
        device_name: &str,
        module_name: &str,
        cache_enable: bool,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            mem_accessor: MemAccessor::<UdmabufRegion, U>::new(
                UdmabufRegion::new_with_module_name(device_name, module_name, cache_enable)?,
            ),
        })
    }

    pub fn subclone_<NewU>(&self, offset: usize, size: usize) -> UdmabufAccessor<NewU> {
        UdmabufAccessor::<NewU> {
            mem_accessor: MemAccessor::<UdmabufRegion, NewU>::new(
                self.mem_accessor.region().subclone(offset, size),
            ),
        }
    }

    pub fn subclone8(&self, offset: usize, size: usize) -> UdmabufAccessor<u8> {
        self.subclone_::<u8>(offset, size)
    }

    pub fn subclone16(&self, offset: usize, size: usize) -> UdmabufAccessor<u16> {
        self.subclone_::<u16>(offset, size)
    }

    pub fn subclone32(&self, offset: usize, size: usize) -> UdmabufAccessor<u32> {
        self.subclone_::<u32>(offset, size)
    }

    pub fn subclone64(&self, offset: usize, size: usize) -> UdmabufAccessor<u64> {
        self.subclone_::<u64>(offset, size)
    }

    delegate! {
        to self.mem_accessor.region() {
            pub fn addr(&self) -> usize;
            pub fn size(&self) -> usize;

            pub fn read_phys_addr(&self) -> Result<usize, Box<dyn Error>>;
            pub fn read_phys_size(&self) -> Result<usize, Box<dyn Error>>;
            pub fn read_sync_mode(&self) -> Result<u32, Box<dyn Error>> ;
            pub fn write_sync_mode(&self, sync_mode: u32) -> Result<(), Box<dyn Error>> ;
            pub fn read_sync_offset(&self) -> Result<usize, Box<dyn Error>> ;
            pub fn write_sync_offset(&self, sync_offset: usize) -> Result<(), Box<dyn Error>> ;
            pub fn read_sync_size(&self) -> Result<usize, Box<dyn Error>> ;
            pub fn write_sync_size(&self, sync_size: usize) -> Result<(), Box<dyn Error>> ;
            pub fn read_sync_direction(&self) -> Result<u32, Box<dyn Error>> ;
            pub fn write_sync_direction(&self, sync_size: usize) -> Result<(), Box<dyn Error>> ;
            pub fn read_dma_coherent(&self) -> Result<u32, Box<dyn Error>> ;
            pub fn read_sync_owner(&self) -> Result<u32, Box<dyn Error>> ;
            pub fn write_sync_for_cpu(&self) -> Result<(), Box<dyn Error>> ;
            pub fn write_sync_for_cpu_with_range(
                &self,
                sync_offset: usize,
                sync_size: usize,
                sync_direction: u32,
                sync_for_cpu: u32,
            ) -> Result<(), Box<dyn Error>> ;

            pub fn write_sync_for_device(&self) -> Result<(), Box<dyn Error>> ;
            pub fn write_sync_for_device_with_range(
                &self,
                sync_offset: usize,
                sync_size: usize,
                sync_direction: u32,
                sync_for_cpu: u32,
            ) -> Result<(), Box<dyn Error>> ;
        }
    }
}

impl<U> Clone for UdmabufAccessor<U> {
    fn clone(&self) -> Self {
        self.subclone(0, 0)
    }
}

impl<U> MemAccessBase for UdmabufAccessor<U> {
    fn reg_size() -> usize {
        core::mem::size_of::<U>()
    }

    fn subclone(&self, offset: usize, size: usize) -> Self {
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

impl<U> MemAccess for UdmabufAccessor<U> {
    delegate! {
        to self.mem_accessor {

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
    }
}

impl<U> MemAccessSync for UdmabufAccessor<U> {
    unsafe fn sync_owner(&self) -> u32 {
        self.mem_accessor.region().read_sync_owner().unwrap()
    }

    unsafe fn sync_for_cpu(&self) {
        self.mem_accessor.region().write_sync_for_cpu().unwrap()
    }

    unsafe fn sync_for_cpu_with_range(
        &self,
        sync_offset: usize,
        sync_size: usize,
        sync_direction: u32,
        sync_for_cpu: u32,
    ) {
        self.mem_accessor
            .region()
            .write_sync_for_cpu_with_range(sync_offset, sync_size, sync_direction, sync_for_cpu)
            .unwrap()
    }

    unsafe fn sync_for_device(&self) {
        self.mem_accessor.region().write_sync_for_device().unwrap();
    }

    unsafe fn sync_for_device_with_range(
        &self,
        sync_offset: usize,
        sync_size: usize,
        sync_direction: u32,
        sync_for_device: u32,
    ) {
        self.mem_accessor
            .region()
            .write_sync_for_device_with_range(
                sync_offset,
                sync_size,
                sync_direction,
                sync_for_device,
            )
            .unwrap()
    }
}
