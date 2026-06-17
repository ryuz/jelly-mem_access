#![allow(dead_code)]

use core::fmt;
use core::marker::PhantomData;

use crate::{MemAccess, MemAccessTryError};

pub trait Bus<A, D, S> {
    type Error;

    fn write(&mut self, addr: A, data: D, strb: S) -> Result<(), Self::Error>;
    fn read(&mut self, addr: A) -> Result<D, Self::Error>;
}

pub trait BusAddress: Copy {
    fn to_usize(self) -> usize;
    fn try_from_usize(value: usize) -> Option<Self>;
}

pub trait BusWord: Copy {
    const BYTES: usize;
    const BITS: usize;

    fn to_u128(self) -> u128;
    fn from_u128(value: u128) -> Self;
}

pub trait BusValue: Copy {
    const BYTES: usize;

    fn to_u128(self) -> u128;
    fn from_u128(value: u128) -> Self;
}

pub trait Endianness {
    fn lane_byte_index(word_bytes: usize, byte_offset_in_word: usize) -> usize;
    fn value_byte_index(value_bytes: usize, byte_offset: usize) -> usize;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LittleEndian;

impl Endianness for LittleEndian {
    fn lane_byte_index(_word_bytes: usize, byte_offset_in_word: usize) -> usize {
        byte_offset_in_word
    }

    fn value_byte_index(_value_bytes: usize, byte_offset: usize) -> usize {
        byte_offset
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BigEndian;

impl Endianness for BigEndian {
    fn lane_byte_index(word_bytes: usize, byte_offset_in_word: usize) -> usize {
        word_bytes - 1 - byte_offset_in_word
    }

    fn value_byte_index(value_bytes: usize, byte_offset: usize) -> usize {
        value_bytes - 1 - byte_offset
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusAccessorError<E> {
    AddressOverflow,
    AddressOutOfRange,
    OutOfBounds,
    StrbTooNarrow,
    Bus(E),
}

impl<E: fmt::Display> fmt::Display for BusAccessorError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddressOverflow => write!(f, "address arithmetic overflow"),
            Self::AddressOutOfRange => write!(f, "address is out of representable range"),
            Self::OutOfBounds => write!(f, "access exceeds region bounds"),
            Self::StrbTooNarrow => write!(f, "strb width is smaller than data byte lanes"),
            Self::Bus(err) => write!(f, "bus access failed: {err}"),
        }
    }
}

#[cfg(feature = "std")]
impl<E> std::error::Error for BusAccessorError<E> where E: std::error::Error + 'static {}

#[derive(Debug)]
pub struct BusAccessor<B, A, D, S, E>
where
    B: Bus<A, D, S>,
{
    bus: B,
    _phantom: PhantomData<(A, D, S, E)>,
}

impl<B, A, D, S, E> BusAccessor<B, A, D, S, E>
where
    B: Bus<A, D, S>,
    A: BusAddress,
    D: BusWord,
    S: BusWord,
    E: Endianness,
{
    pub fn new(bus: B) -> Self {
        Self {
            bus,
            _phantom: PhantomData,
        }
    }

    pub fn bus(&self) -> &B {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut B {
        &mut self.bus
    }

    pub fn into_inner(self) -> B {
        self.bus
    }

    pub fn write_value<V: BusValue>(
        &mut self,
        addr: A,
        value: V,
    ) -> Result<(), BusAccessorError<B::Error>> {
        if S::BITS < D::BYTES {
            return Err(BusAccessorError::StrbTooNarrow);
        }

        let word_bytes = D::BYTES;
        let total_bytes = V::BYTES;
        let mut processed = 0usize;
        let base_addr = addr.to_usize();
        let value_bits = value.to_u128();

        while processed < total_bytes {
            let cur_addr = base_addr
                .checked_add(processed)
                .ok_or(BusAccessorError::AddressOverflow)?;
            let word_addr = (cur_addr / word_bytes) * word_bytes;
            let lane_offset = cur_addr - word_addr;
            let chunk_bytes = core::cmp::min(word_bytes - lane_offset, total_bytes - processed);

            let mut data_word = 0u128;
            let mut strb_word = 0u128;

            for i in 0..chunk_bytes {
                let value_mem_offset = processed + i;
                let value_byte_index = E::value_byte_index(total_bytes, value_mem_offset);
                let value_byte = ((value_bits >> (value_byte_index * 8)) & 0xFF) as u8;

                let lane = E::lane_byte_index(word_bytes, lane_offset + i);
                data_word |= (value_byte as u128) << (lane * 8);
                strb_word |= 1u128 << lane;
            }

            let write_addr = A::try_from_usize(word_addr).ok_or(BusAccessorError::AddressOutOfRange)?;
            self.bus
                .write(write_addr, D::from_u128(data_word), S::from_u128(strb_word))
                .map_err(BusAccessorError::Bus)?;

            processed += chunk_bytes;
        }

        Ok(())
    }

    pub fn read_value<V: BusValue>(&mut self, addr: A) -> Result<V, BusAccessorError<B::Error>> {
        if S::BITS < D::BYTES {
            return Err(BusAccessorError::StrbTooNarrow);
        }

        let word_bytes = D::BYTES;
        let total_bytes = V::BYTES;
        let mut processed = 0usize;
        let base_addr = addr.to_usize();
        let mut value_bits = 0u128;

        while processed < total_bytes {
            let cur_addr = base_addr
                .checked_add(processed)
                .ok_or(BusAccessorError::AddressOverflow)?;
            let word_addr = (cur_addr / word_bytes) * word_bytes;
            let lane_offset = cur_addr - word_addr;
            let chunk_bytes = core::cmp::min(word_bytes - lane_offset, total_bytes - processed);

            let read_addr = A::try_from_usize(word_addr).ok_or(BusAccessorError::AddressOutOfRange)?;
            let word = self.bus.read(read_addr).map_err(BusAccessorError::Bus)?;
            let word_bits = word.to_u128();

            for i in 0..chunk_bytes {
                let lane = E::lane_byte_index(word_bytes, lane_offset + i);
                let value_mem_offset = processed + i;
                let value_byte_index = E::value_byte_index(total_bytes, value_mem_offset);
                let lane_byte = ((word_bits >> (lane * 8)) & 0xFF) as u8;
                value_bits |= (lane_byte as u128) << (value_byte_index * 8);
            }

            processed += chunk_bytes;
        }

        Ok(V::from_u128(value_bits))
    }

    pub fn write_u8(&mut self, addr: A, value: u8) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(addr, value)
    }

    pub fn write_u16(&mut self, addr: A, value: u16) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(addr, value)
    }

    pub fn write_u32(&mut self, addr: A, value: u32) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(addr, value)
    }

    pub fn write_u64(&mut self, addr: A, value: u64) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(addr, value)
    }

    pub fn write_usize(
        &mut self,
        addr: A,
        value: usize,
    ) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(addr, value)
    }

    pub fn read_u8(&mut self, addr: A) -> Result<u8, BusAccessorError<B::Error>> {
        self.read_value(addr)
    }

    pub fn read_u16(&mut self, addr: A) -> Result<u16, BusAccessorError<B::Error>> {
        self.read_value(addr)
    }

    pub fn read_u32(&mut self, addr: A) -> Result<u32, BusAccessorError<B::Error>> {
        self.read_value(addr)
    }

    pub fn read_u64(&mut self, addr: A) -> Result<u64, BusAccessorError<B::Error>> {
        self.read_value(addr)
    }

    pub fn read_usize(&mut self, addr: A) -> Result<usize, BusAccessorError<B::Error>> {
        self.read_value(addr)
    }
}

macro_rules! impl_bus_address {
    ($t:ty) => {
        impl BusAddress for $t {
            fn to_usize(self) -> usize {
                self as usize
            }

            fn try_from_usize(value: usize) -> Option<Self> {
                if value <= <$t>::MAX as usize {
                    Some(value as $t)
                } else {
                    None
                }
            }
        }
    };
}

impl_bus_address!(u8);
impl_bus_address!(u16);
impl_bus_address!(u32);
impl_bus_address!(u64);
impl BusAddress for usize {
    fn to_usize(self) -> usize {
        self
    }

    fn try_from_usize(value: usize) -> Option<Self> {
        Some(value)
    }
}

macro_rules! impl_bus_word {
    ($t:ty) => {
        impl BusWord for $t {
            const BYTES: usize = core::mem::size_of::<$t>();
            const BITS: usize = core::mem::size_of::<$t>() * 8;

            fn to_u128(self) -> u128 {
                self as u128
            }

            fn from_u128(value: u128) -> Self {
                value as $t
            }
        }
    };
}

impl_bus_word!(u8);
impl_bus_word!(u16);
impl_bus_word!(u32);
impl_bus_word!(u64);
impl_bus_word!(u128);
impl_bus_word!(usize);

macro_rules! impl_bus_value_unsigned {
    ($t:ty) => {
        impl BusValue for $t {
            const BYTES: usize = core::mem::size_of::<$t>();

            fn to_u128(self) -> u128 {
                self as u128
            }

            fn from_u128(value: u128) -> Self {
                value as $t
            }
        }
    };
}

macro_rules! impl_bus_value_signed {
    ($t:ty, $u:ty) => {
        impl BusValue for $t {
            const BYTES: usize = core::mem::size_of::<$t>();

            fn to_u128(self) -> u128 {
                (self as $u) as u128
            }

            fn from_u128(value: u128) -> Self {
                (value as $u) as $t
            }
        }
    };
}

impl_bus_value_unsigned!(u8);
impl_bus_value_unsigned!(u16);
impl_bus_value_unsigned!(u32);
impl_bus_value_unsigned!(u64);
impl_bus_value_unsigned!(u128);
impl_bus_value_unsigned!(usize);

impl_bus_value_signed!(i8, u8);
impl_bus_value_signed!(i16, u16);
impl_bus_value_signed!(i32, u32);
impl_bus_value_signed!(i64, u64);
impl_bus_value_signed!(i128, u128);
impl_bus_value_signed!(isize, usize);

impl BusValue for f32 {
    const BYTES: usize = core::mem::size_of::<f32>();

    fn to_u128(self) -> u128 {
        self.to_bits() as u128
    }

    fn from_u128(value: u128) -> Self {
        Self::from_bits(value as u32)
    }
}

impl BusValue for f64 {
    const BYTES: usize = core::mem::size_of::<f64>();

    fn to_u128(self) -> u128 {
        self.to_bits() as u128
    }

    fn from_u128(value: u128) -> Self {
        Self::from_bits(value as u64)
    }
}

fn map_bus_err<E>(err: BusAccessorError<E>) -> MemAccessTryError {
    match err {
        BusAccessorError::AddressOverflow => MemAccessTryError::AddressOverflow,
        BusAccessorError::AddressOutOfRange => MemAccessTryError::AddressOutOfRange,
        BusAccessorError::OutOfBounds => MemAccessTryError::OutOfBounds,
        BusAccessorError::StrbTooNarrow => MemAccessTryError::StrbTooNarrow,
        BusAccessorError::Bus(_) => MemAccessTryError::AccessFault,
    }
}

impl<B, A, D, S, E> MemAccess for BusAccessor<B, A, D, S, E>
where
    B: Bus<A, D, S>,
    A: BusAddress,
    D: BusWord,
    S: BusWord,
    E: Endianness,
{
    fn addr(&self) -> usize { 0 }
    fn size(&self) -> usize { 0 }
    fn phys_addr(&self) -> usize { 0 }

    unsafe fn copy_to_usize(&self, src_adr: usize, dst_ptr: *mut usize, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_usize(src_adr + i * core::mem::size_of::<usize>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_u8(&self, src_adr: usize, dst_ptr: *mut u8, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_u8(src_adr + i) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_u16(&self, src_adr: usize, dst_ptr: *mut u16, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_u16(src_adr + i * core::mem::size_of::<u16>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_u32(&self, src_adr: usize, dst_ptr: *mut u32, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_u32(src_adr + i * core::mem::size_of::<u32>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_u64(&self, src_adr: usize, dst_ptr: *mut u64, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_u64(src_adr + i * core::mem::size_of::<u64>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_isize(&self, src_adr: usize, dst_ptr: *mut isize, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_isize(src_adr + i * core::mem::size_of::<isize>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_i8(&self, src_adr: usize, dst_ptr: *mut i8, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_i8(src_adr + i) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_i16(&self, src_adr: usize, dst_ptr: *mut i16, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_i16(src_adr + i * core::mem::size_of::<i16>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_i32(&self, src_adr: usize, dst_ptr: *mut i32, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_i32(src_adr + i * core::mem::size_of::<i32>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_i64(&self, src_adr: usize, dst_ptr: *mut i64, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_i64(src_adr + i * core::mem::size_of::<i64>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_f32(&self, src_adr: usize, dst_ptr: *mut f32, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_f32(src_adr + i * core::mem::size_of::<f32>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }
    unsafe fn copy_to_f64(&self, src_adr: usize, dst_ptr: *mut f64, count: usize) {
        for i in 0..count {
            let v = unsafe { self.read_mem_f64(src_adr + i * core::mem::size_of::<f64>()) };
            unsafe { core::ptr::write(dst_ptr.add(i), v) };
        }
    }

    unsafe fn copy_from_usize(&self, src_ptr: *const usize, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_usize(dst_adr + i * core::mem::size_of::<usize>(), v) };
        }
    }
    unsafe fn copy_from_u8(&self, src_ptr: *const u8, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_u8(dst_adr + i, v) };
        }
    }
    unsafe fn copy_from_u16(&self, src_ptr: *const u16, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_u16(dst_adr + i * core::mem::size_of::<u16>(), v) };
        }
    }
    unsafe fn copy_from_u32(&self, src_ptr: *const u32, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_u32(dst_adr + i * core::mem::size_of::<u32>(), v) };
        }
    }
    unsafe fn copy_from_u64(&self, src_ptr: *const u64, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_u64(dst_adr + i * core::mem::size_of::<u64>(), v) };
        }
    }
    unsafe fn copy_from_isize(&self, src_ptr: *const isize, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_isize(dst_adr + i * core::mem::size_of::<isize>(), v) };
        }
    }
    unsafe fn copy_from_i8(&self, src_ptr: *const i8, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_i8(dst_adr + i, v) };
        }
    }
    unsafe fn copy_from_i16(&self, src_ptr: *const i16, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_i16(dst_adr + i * core::mem::size_of::<i16>(), v) };
        }
    }
    unsafe fn copy_from_i32(&self, src_ptr: *const i32, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_i32(dst_adr + i * core::mem::size_of::<i32>(), v) };
        }
    }
    unsafe fn copy_from_i64(&self, src_ptr: *const i64, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_i64(dst_adr + i * core::mem::size_of::<i64>(), v) };
        }
    }
    unsafe fn copy_from_f32(&self, src_ptr: *const f32, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_f32(dst_adr + i * core::mem::size_of::<f32>(), v) };
        }
    }
    unsafe fn copy_from_f64(&self, src_ptr: *const f64, dst_adr: usize, count: usize) {
        for i in 0..count {
            let v = unsafe { core::ptr::read(src_ptr.add(i)) };
            unsafe { self.write_mem_f64(dst_adr + i * core::mem::size_of::<f64>(), v) };
        }
    }

    unsafe fn write_mem(&self, offset: usize, data: usize) { unsafe { self.try_write_mem(offset, data) }.unwrap(); }
    unsafe fn write_mem_usize(&self, offset: usize, data: usize) { unsafe { self.try_write_mem_usize(offset, data) }.unwrap(); }
    unsafe fn write_mem_u8(&self, offset: usize, data: u8) { unsafe { self.try_write_mem_u8(offset, data) }.unwrap(); }
    unsafe fn write_mem_u16(&self, offset: usize, data: u16) { unsafe { self.try_write_mem_u16(offset, data) }.unwrap(); }
    unsafe fn write_mem_u32(&self, offset: usize, data: u32) { unsafe { self.try_write_mem_u32(offset, data) }.unwrap(); }
    unsafe fn write_mem_u64(&self, offset: usize, data: u64) { unsafe { self.try_write_mem_u64(offset, data) }.unwrap(); }
    unsafe fn write_mem_isize(&self, offset: usize, data: isize) { unsafe { self.try_write_mem_isize(offset, data) }.unwrap(); }
    unsafe fn write_mem_i8(&self, offset: usize, data: i8) { unsafe { self.try_write_mem_i8(offset, data) }.unwrap(); }
    unsafe fn write_mem_i16(&self, offset: usize, data: i16) { unsafe { self.try_write_mem_i16(offset, data) }.unwrap(); }
    unsafe fn write_mem_i32(&self, offset: usize, data: i32) { unsafe { self.try_write_mem_i32(offset, data) }.unwrap(); }
    unsafe fn write_mem_i64(&self, offset: usize, data: i64) { unsafe { self.try_write_mem_i64(offset, data) }.unwrap(); }
    unsafe fn write_mem_f32(&self, offset: usize, data: f32) { unsafe { self.try_write_mem_f32(offset, data) }.unwrap(); }
    unsafe fn write_mem_f64(&self, offset: usize, data: f64) { unsafe { self.try_write_mem_f64(offset, data) }.unwrap(); }

    unsafe fn read_mem(&self, offset: usize) -> usize { unsafe { self.try_read_mem(offset) }.unwrap() }
    unsafe fn read_mem_usize(&self, offset: usize) -> usize { unsafe { self.try_read_mem_usize(offset) }.unwrap() }
    unsafe fn read_mem_u8(&self, offset: usize) -> u8 { unsafe { self.try_read_mem_u8(offset) }.unwrap() }
    unsafe fn read_mem_u16(&self, offset: usize) -> u16 { unsafe { self.try_read_mem_u16(offset) }.unwrap() }
    unsafe fn read_mem_u32(&self, offset: usize) -> u32 { unsafe { self.try_read_mem_u32(offset) }.unwrap() }
    unsafe fn read_mem_u64(&self, offset: usize) -> u64 { unsafe { self.try_read_mem_u64(offset) }.unwrap() }
    unsafe fn read_mem_isize(&self, offset: usize) -> isize { unsafe { self.try_read_mem_isize(offset) }.unwrap() }
    unsafe fn read_mem_i8(&self, offset: usize) -> i8 { unsafe { self.try_read_mem_i8(offset) }.unwrap() }
    unsafe fn read_mem_i16(&self, offset: usize) -> i16 { unsafe { self.try_read_mem_i16(offset) }.unwrap() }
    unsafe fn read_mem_i32(&self, offset: usize) -> i32 { unsafe { self.try_read_mem_i32(offset) }.unwrap() }
    unsafe fn read_mem_i64(&self, offset: usize) -> i64 { unsafe { self.try_read_mem_i64(offset) }.unwrap() }
    unsafe fn read_mem_f32(&self, offset: usize) -> f32 { unsafe { self.try_read_mem_f32(offset) }.unwrap() }
    unsafe fn read_mem_f64(&self, offset: usize) -> f64 { unsafe { self.try_read_mem_f64(offset) }.unwrap() }

    unsafe fn write_reg(&self, reg: usize, data: usize) { unsafe { self.write_mem(reg * D::BYTES, data) } }
    unsafe fn write_reg_usize(&self, reg: usize, data: usize) { unsafe { self.write_mem_usize(reg * D::BYTES, data) } }
    unsafe fn write_reg_u8(&self, reg: usize, data: u8) { unsafe { self.write_mem_u8(reg * D::BYTES, data) } }
    unsafe fn write_reg_u16(&self, reg: usize, data: u16) { unsafe { self.write_mem_u16(reg * D::BYTES, data) } }
    unsafe fn write_reg_u32(&self, reg: usize, data: u32) { unsafe { self.write_mem_u32(reg * D::BYTES, data) } }
    unsafe fn write_reg_u64(&self, reg: usize, data: u64) { unsafe { self.write_mem_u64(reg * D::BYTES, data) } }
    unsafe fn write_reg_isize(&self, reg: usize, data: isize) { unsafe { self.write_mem_isize(reg * D::BYTES, data) } }
    unsafe fn write_reg_i8(&self, reg: usize, data: i8) { unsafe { self.write_mem_i8(reg * D::BYTES, data) } }
    unsafe fn write_reg_i16(&self, reg: usize, data: i16) { unsafe { self.write_mem_i16(reg * D::BYTES, data) } }
    unsafe fn write_reg_i32(&self, reg: usize, data: i32) { unsafe { self.write_mem_i32(reg * D::BYTES, data) } }
    unsafe fn write_reg_i64(&self, reg: usize, data: i64) { unsafe { self.write_mem_i64(reg * D::BYTES, data) } }
    unsafe fn write_reg_f32(&self, reg: usize, data: f32) { unsafe { self.write_mem_f32(reg * D::BYTES, data) } }
    unsafe fn write_reg_f64(&self, reg: usize, data: f64) { unsafe { self.write_mem_f64(reg * D::BYTES, data) } }

    unsafe fn read_reg(&self, reg: usize) -> usize { unsafe { self.read_mem(reg * D::BYTES) } }
    unsafe fn read_reg_usize(&self, reg: usize) -> usize { unsafe { self.read_mem_usize(reg * D::BYTES) } }
    unsafe fn read_reg_u8(&self, reg: usize) -> u8 { unsafe { self.read_mem_u8(reg * D::BYTES) } }
    unsafe fn read_reg_u16(&self, reg: usize) -> u16 { unsafe { self.read_mem_u16(reg * D::BYTES) } }
    unsafe fn read_reg_u32(&self, reg: usize) -> u32 { unsafe { self.read_mem_u32(reg * D::BYTES) } }
    unsafe fn read_reg_u64(&self, reg: usize) -> u64 { unsafe { self.read_mem_u64(reg * D::BYTES) } }
    unsafe fn read_reg_isize(&self, reg: usize) -> isize { unsafe { self.read_mem_isize(reg * D::BYTES) } }
    unsafe fn read_reg_i8(&self, reg: usize) -> i8 { unsafe { self.read_mem_i8(reg * D::BYTES) } }
    unsafe fn read_reg_i16(&self, reg: usize) -> i16 { unsafe { self.read_mem_i16(reg * D::BYTES) } }
    unsafe fn read_reg_i32(&self, reg: usize) -> i32 { unsafe { self.read_mem_i32(reg * D::BYTES) } }
    unsafe fn read_reg_i64(&self, reg: usize) -> i64 { unsafe { self.read_mem_i64(reg * D::BYTES) } }
    unsafe fn read_reg_f32(&self, reg: usize) -> f32 { unsafe { self.read_mem_f32(reg * D::BYTES) } }
    unsafe fn read_reg_f64(&self, reg: usize) -> f64 { unsafe { self.read_mem_f64(reg * D::BYTES) } }

    unsafe fn try_write_mem(&self, offset: usize, data: usize) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_usize(&self, offset: usize, data: usize) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_u8(&self, offset: usize, data: u8) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_u16(&self, offset: usize, data: u16) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_u32(&self, offset: usize, data: u32) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_u64(&self, offset: usize, data: u64) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_isize(&self, offset: usize, data: isize) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_i8(&self, offset: usize, data: i8) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_i16(&self, offset: usize, data: i16) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_i32(&self, offset: usize, data: i32) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_i64(&self, offset: usize, data: i64) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_f32(&self, offset: usize, data: f32) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }
    unsafe fn try_write_mem_f64(&self, offset: usize, data: f64) -> Result<(), MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).write_value(addr, data) }.map_err(map_bus_err)
    }

    unsafe fn try_read_mem(&self, offset: usize) -> Result<usize, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_usize(&self, offset: usize) -> Result<usize, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_u8(&self, offset: usize) -> Result<u8, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_u16(&self, offset: usize) -> Result<u16, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_u32(&self, offset: usize) -> Result<u32, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_u64(&self, offset: usize) -> Result<u64, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_isize(&self, offset: usize) -> Result<isize, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_i8(&self, offset: usize) -> Result<i8, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_i16(&self, offset: usize) -> Result<i16, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_i32(&self, offset: usize) -> Result<i32, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_i64(&self, offset: usize) -> Result<i64, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_f32(&self, offset: usize) -> Result<f32, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }
    unsafe fn try_read_mem_f64(&self, offset: usize) -> Result<f64, MemAccessTryError> {
        let this = self as *const Self as *mut Self;
        let addr = A::try_from_usize(offset).ok_or(MemAccessTryError::AddressOutOfRange)?;
        unsafe { (*this).read_value(addr) }.map_err(map_bus_err)
    }

    unsafe fn try_write_reg(&self, reg: usize, data: usize) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem(offset, data) }
    }
    unsafe fn try_write_reg_usize(&self, reg: usize, data: usize) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_usize(offset, data) }
    }
    unsafe fn try_write_reg_u8(&self, reg: usize, data: u8) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_u8(offset, data) }
    }
    unsafe fn try_write_reg_u16(&self, reg: usize, data: u16) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_u16(offset, data) }
    }
    unsafe fn try_write_reg_u32(&self, reg: usize, data: u32) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_u32(offset, data) }
    }
    unsafe fn try_write_reg_u64(&self, reg: usize, data: u64) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_u64(offset, data) }
    }
    unsafe fn try_write_reg_isize(&self, reg: usize, data: isize) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_isize(offset, data) }
    }
    unsafe fn try_write_reg_i8(&self, reg: usize, data: i8) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_i8(offset, data) }
    }
    unsafe fn try_write_reg_i16(&self, reg: usize, data: i16) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_i16(offset, data) }
    }
    unsafe fn try_write_reg_i32(&self, reg: usize, data: i32) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_i32(offset, data) }
    }
    unsafe fn try_write_reg_i64(&self, reg: usize, data: i64) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_i64(offset, data) }
    }
    unsafe fn try_write_reg_f32(&self, reg: usize, data: f32) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_f32(offset, data) }
    }
    unsafe fn try_write_reg_f64(&self, reg: usize, data: f64) -> Result<(), MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_write_mem_f64(offset, data) }
    }

    unsafe fn try_read_reg(&self, reg: usize) -> Result<usize, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem(offset) }
    }
    unsafe fn try_read_reg_usize(&self, reg: usize) -> Result<usize, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_usize(offset) }
    }
    unsafe fn try_read_reg_u8(&self, reg: usize) -> Result<u8, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_u8(offset) }
    }
    unsafe fn try_read_reg_u16(&self, reg: usize) -> Result<u16, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_u16(offset) }
    }
    unsafe fn try_read_reg_u32(&self, reg: usize) -> Result<u32, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_u32(offset) }
    }
    unsafe fn try_read_reg_u64(&self, reg: usize) -> Result<u64, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_u64(offset) }
    }
    unsafe fn try_read_reg_isize(&self, reg: usize) -> Result<isize, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_isize(offset) }
    }
    unsafe fn try_read_reg_i8(&self, reg: usize) -> Result<i8, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_i8(offset) }
    }
    unsafe fn try_read_reg_i16(&self, reg: usize) -> Result<i16, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_i16(offset) }
    }
    unsafe fn try_read_reg_i32(&self, reg: usize) -> Result<i32, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_i32(offset) }
    }
    unsafe fn try_read_reg_i64(&self, reg: usize) -> Result<i64, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_i64(offset) }
    }
    unsafe fn try_read_reg_f32(&self, reg: usize) -> Result<f32, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_f32(offset) }
    }
    unsafe fn try_read_reg_f64(&self, reg: usize) -> Result<f64, MemAccessTryError> {
        let offset = reg.checked_mul(D::BYTES).ok_or(MemAccessTryError::AddressOverflow)?;
        unsafe { self.try_read_mem_f64(offset) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockBus {
        mem: [u8; 32],
        last_addr: usize,
        last_data: u32,
        last_strb: u8,
    }

    impl MockBus {
        fn new() -> Self {
            Self {
                mem: [0; 32],
                last_addr: 0,
                last_data: 0,
                last_strb: 0,
            }
        }
    }

    impl Bus<usize, u32, u8> for MockBus {
        type Error = ();

        fn write(&mut self, addr: usize, data: u32, strb: u8) -> Result<(), Self::Error> {
            self.last_addr = addr;
            self.last_data = data;
            self.last_strb = strb;
            for lane in 0..4 {
                if ((strb >> lane) & 1) == 1 {
                    self.mem[addr + lane] = ((data >> (lane * 8)) & 0xFF) as u8;
                }
            }
            Ok(())
        }

        fn read(&mut self, addr: usize) -> Result<u32, Self::Error> {
            let mut data = 0u32;
            for lane in 0..4 {
                data |= (self.mem[addr + lane] as u32) << (lane * 8);
            }
            Ok(data)
        }
    }

    #[test]
    fn little_endian_subword_access() {
        let bus = MockBus::new();
        let mut accessor = BusAccessor::<_, usize, u32, u8, LittleEndian>::new(bus);

        accessor.write_u16(1, 0xABCD).unwrap();
        let bus = accessor.into_inner();

        assert_eq!(bus.last_addr, 0);
        assert_eq!(bus.last_strb, 0b0110);
        assert_eq!(bus.last_data, 0x00ABCD00);
        assert_eq!(bus.mem[1], 0xCD);
        assert_eq!(bus.mem[2], 0xAB);
    }

    #[test]
    fn big_endian_subword_access() {
        let bus = MockBus::new();
        let mut accessor = BusAccessor::<_, usize, u32, u8, BigEndian>::new(bus);

        accessor.write_u16(0, 0xABCD).unwrap();
        let bus = accessor.into_inner();

        assert_eq!(bus.last_addr, 0);
        assert_eq!(bus.last_strb, 0b1100);
        assert_eq!(bus.last_data, 0xABCD0000);
    }

    #[test]
    fn little_endian_larger_than_bus_word() {
        let bus = MockBus::new();
        let mut accessor = BusAccessor::<_, usize, u32, u8, LittleEndian>::new(bus);

        accessor.write_u64(2, 0x0123_4567_89AB_CDEF).unwrap();
        let value = accessor.read_u64(2).unwrap();
        assert_eq!(value, 0x0123_4567_89AB_CDEF);
    }

    #[test]
    fn big_endian_larger_than_bus_word() {
        let bus = MockBus::new();
        let mut accessor = BusAccessor::<_, usize, u32, u8, BigEndian>::new(bus);

        accessor.write_u64(2, 0x0123_4567_89AB_CDEF).unwrap();
        let value = accessor.read_u64(2).unwrap();
        assert_eq!(value, 0x0123_4567_89AB_CDEF);
    }
}