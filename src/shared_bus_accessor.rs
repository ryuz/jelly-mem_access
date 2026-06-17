#![allow(dead_code)]

use core::marker::PhantomData;
use std::sync::{Arc, Mutex};

use super::bus_accessor::{Bus, BusAccessorError, BusAddress, BusValue, BusWord, Endianness};
use super::{MemAccess, MemAccessTryError};

/// `Bus` の所有権を `Arc<Mutex<B>>` で共有し、`subclone` によるサブリージョン分割が
/// できるバスアクセサ。
///
/// - `base`: このアクセサの先頭アドレス（バス絶対アドレス）
/// - `size`: リージョンサイズ（0 = 無制限）
/// - エンディアンは型パラメータ `E` で静的に決定
/// - `write_value`/`read_value` は `&self` で呼べる（Mutex による内部可変性）
/// - Mutex は書き込み・読み込み共に単一のワード列全体をまとめてロックする
#[derive(Debug)]
pub struct SharedBusAccessor<B, A, D, S, E>
where
    B: Bus<A, D, S>,
{
    bus: Arc<Mutex<B>>,
    base: usize,
    size: usize,
    _phantom: PhantomData<(A, D, S, E)>,
}

// Arc::clone するだけなので B: Clone は不要
impl<B, A, D, S, E> Clone for SharedBusAccessor<B, A, D, S, E>
where
    B: Bus<A, D, S>,
{
    fn clone(&self) -> Self {
        Self {
            bus: Arc::clone(&self.bus),
            base: self.base,
            size: self.size,
            _phantom: PhantomData,
        }
    }
}

impl<B, A, D, S, E> SharedBusAccessor<B, A, D, S, E>
where
    B: Bus<A, D, S>,
    A: BusAddress,
    D: BusWord,
    S: BusWord,
    E: Endianness,
{
    /// `bus` を所有して `SharedBusAccessor` を作る。base=0、size=無制限。
    pub fn new(bus: B) -> Self {
        Self {
            bus: Arc::new(Mutex::new(bus)),
            base: 0,
            size: 0,
            _phantom: PhantomData,
        }
    }

    /// `bus` を所有して指定の `base`/`size` で `SharedBusAccessor` を作る。
    pub fn new_with_range(bus: B, base: usize, size: usize) -> Self {
        Self {
            bus: Arc::new(Mutex::new(bus)),
            base,
            size,
            _phantom: PhantomData,
        }
    }

    /// このアクセサの先頭アドレス（バス絶対アドレス）を返す。
    pub fn base(&self) -> usize {
        self.base
    }

    /// このアクセサのリージョンサイズを返す（0 = 無制限）。
    pub fn size(&self) -> usize {
        self.size
    }

    /// エンディアン型パラメータを変えてサブクローンを作る汎用版。
    ///
    /// - `offset`: このアクセサ先頭からのバイトオフセット
    /// - `size`: サブリージョンのサイズ（0 = 残り全部）
    pub fn subclone_<NewE: Endianness>(&self, offset: usize, size: usize) -> SharedBusAccessor<B, A, D, S, NewE> {
        debug_assert!(self.size == 0 || offset <= self.size);
        let new_base = self.base.saturating_add(offset);
        let new_size = if self.size == 0 {
            size
        } else {
            let available = self.size.saturating_sub(offset);
            if size == 0 { available } else { size.min(available) }
        };
        SharedBusAccessor {
            bus: Arc::clone(&self.bus),
            base: new_base,
            size: new_size,
            _phantom: PhantomData,
        }
    }

    /// 同じエンディアンでサブクローンを作る。
    pub fn subclone(&self, offset: usize, size: usize) -> Self {
        self.subclone_::<E>(offset, size)
    }

    // -----------------------------------------------------------------------
    //  コアアクセス
    // -----------------------------------------------------------------------

    /// 任意の `BusValue` 型を `offset` バイト位置に書き込む。
    /// データ幅より小さい場合は strb で部分書き込み、大きい場合は複数アクセスに分割する。
    pub fn write_value<V: BusValue>(
        &self,
        offset: usize,
        value: V,
    ) -> Result<(), BusAccessorError<B::Error>> {
        if S::BITS < D::BYTES {
            return Err(BusAccessorError::StrbTooNarrow);
        }
        if self.size != 0 {
            let end = offset.checked_add(V::BYTES).ok_or(BusAccessorError::AddressOverflow)?;
            if end > self.size {
                return Err(BusAccessorError::OutOfBounds);
            }
        }

        let word_bytes = D::BYTES;
        let total_bytes = V::BYTES;
        let abs_base = self.base.checked_add(offset).ok_or(BusAccessorError::AddressOverflow)?;
        let value_bits = value.to_u128();
        let mut processed = 0usize;

        let mut bus = self.bus.lock().unwrap();

        while processed < total_bytes {
            let cur_addr = abs_base
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
            bus.write(write_addr, D::from_u128(data_word), S::from_u128(strb_word))
                .map_err(BusAccessorError::Bus)?;

            processed += chunk_bytes;
        }

        Ok(())
    }

    /// 任意の `BusValue` 型を `offset` バイト位置から読み込む。
    /// データ幅より小さい/大きい場合の処理は `write_value` と対称。
    pub fn read_value<V: BusValue>(
        &self,
        offset: usize,
    ) -> Result<V, BusAccessorError<B::Error>> {
        if S::BITS < D::BYTES {
            return Err(BusAccessorError::StrbTooNarrow);
        }
        if self.size != 0 {
            let end = offset.checked_add(V::BYTES).ok_or(BusAccessorError::AddressOverflow)?;
            if end > self.size {
                return Err(BusAccessorError::OutOfBounds);
            }
        }

        let word_bytes = D::BYTES;
        let total_bytes = V::BYTES;
        let abs_base = self.base.checked_add(offset).ok_or(BusAccessorError::AddressOverflow)?;
        let mut value_bits = 0u128;
        let mut processed = 0usize;

        let mut bus = self.bus.lock().unwrap();

        while processed < total_bytes {
            let cur_addr = abs_base
                .checked_add(processed)
                .ok_or(BusAccessorError::AddressOverflow)?;
            let word_addr = (cur_addr / word_bytes) * word_bytes;
            let lane_offset = cur_addr - word_addr;
            let chunk_bytes = core::cmp::min(word_bytes - lane_offset, total_bytes - processed);

            let read_addr = A::try_from_usize(word_addr).ok_or(BusAccessorError::AddressOutOfRange)?;
            let word = bus.read(read_addr).map_err(BusAccessorError::Bus)?;
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

    // -----------------------------------------------------------------------
    //  型付きアクセス便利メソッド
    // -----------------------------------------------------------------------

    pub fn write_u8(&self, offset: usize, value: u8) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(offset, value)
    }

    pub fn write_u16(&self, offset: usize, value: u16) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(offset, value)
    }

    pub fn write_u32(&self, offset: usize, value: u32) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(offset, value)
    }

    pub fn write_u64(&self, offset: usize, value: u64) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(offset, value)
    }

    pub fn write_usize(&self, offset: usize, value: usize) -> Result<(), BusAccessorError<B::Error>> {
        self.write_value(offset, value)
    }

    pub fn read_u8(&self, offset: usize) -> Result<u8, BusAccessorError<B::Error>> {
        self.read_value(offset)
    }

    pub fn read_u16(&self, offset: usize) -> Result<u16, BusAccessorError<B::Error>> {
        self.read_value(offset)
    }

    pub fn read_u32(&self, offset: usize) -> Result<u32, BusAccessorError<B::Error>> {
        self.read_value(offset)
    }

    pub fn read_u64(&self, offset: usize) -> Result<u64, BusAccessorError<B::Error>> {
        self.read_value(offset)
    }

    pub fn read_usize(&self, offset: usize) -> Result<usize, BusAccessorError<B::Error>> {
        self.read_value(offset)
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

impl<B, A, D, S, E> MemAccess for SharedBusAccessor<B, A, D, S, E>
where
    B: Bus<A, D, S>,
    A: BusAddress,
    D: BusWord,
    S: BusWord,
    E: Endianness,
{
    fn addr(&self) -> usize { self.base }
    fn size(&self) -> usize { self.size }
    fn phys_addr(&self) -> usize { self.base }

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
        self.write_usize(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_usize(&self, offset: usize, data: usize) -> Result<(), MemAccessTryError> {
        self.write_usize(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_u8(&self, offset: usize, data: u8) -> Result<(), MemAccessTryError> {
        self.write_u8(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_u16(&self, offset: usize, data: u16) -> Result<(), MemAccessTryError> {
        self.write_u16(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_u32(&self, offset: usize, data: u32) -> Result<(), MemAccessTryError> {
        self.write_u32(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_u64(&self, offset: usize, data: u64) -> Result<(), MemAccessTryError> {
        self.write_u64(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_isize(&self, offset: usize, data: isize) -> Result<(), MemAccessTryError> {
        self.write_value(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_i8(&self, offset: usize, data: i8) -> Result<(), MemAccessTryError> {
        self.write_value(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_i16(&self, offset: usize, data: i16) -> Result<(), MemAccessTryError> {
        self.write_value(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_i32(&self, offset: usize, data: i32) -> Result<(), MemAccessTryError> {
        self.write_value(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_i64(&self, offset: usize, data: i64) -> Result<(), MemAccessTryError> {
        self.write_value(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_f32(&self, offset: usize, data: f32) -> Result<(), MemAccessTryError> {
        self.write_value(offset, data).map_err(map_bus_err)
    }
    unsafe fn try_write_mem_f64(&self, offset: usize, data: f64) -> Result<(), MemAccessTryError> {
        self.write_value(offset, data).map_err(map_bus_err)
    }

    unsafe fn try_read_mem(&self, offset: usize) -> Result<usize, MemAccessTryError> {
        self.read_usize(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_usize(&self, offset: usize) -> Result<usize, MemAccessTryError> {
        self.read_usize(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_u8(&self, offset: usize) -> Result<u8, MemAccessTryError> {
        self.read_u8(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_u16(&self, offset: usize) -> Result<u16, MemAccessTryError> {
        self.read_u16(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_u32(&self, offset: usize) -> Result<u32, MemAccessTryError> {
        self.read_u32(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_u64(&self, offset: usize) -> Result<u64, MemAccessTryError> {
        self.read_u64(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_isize(&self, offset: usize) -> Result<isize, MemAccessTryError> {
        self.read_value(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_i8(&self, offset: usize) -> Result<i8, MemAccessTryError> {
        self.read_value(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_i16(&self, offset: usize) -> Result<i16, MemAccessTryError> {
        self.read_value(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_i32(&self, offset: usize) -> Result<i32, MemAccessTryError> {
        self.read_value(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_i64(&self, offset: usize) -> Result<i64, MemAccessTryError> {
        self.read_value(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_f32(&self, offset: usize) -> Result<f32, MemAccessTryError> {
        self.read_value(offset).map_err(map_bus_err)
    }
    unsafe fn try_read_mem_f64(&self, offset: usize) -> Result<f64, MemAccessTryError> {
        self.read_value(offset).map_err(map_bus_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus_accessor::{BigEndian, LittleEndian};

    #[derive(Debug)]
    struct MockBus {
        mem: [u8; 64],
    }

    impl Default for MockBus {
        fn default() -> Self {
            Self { mem: [0; 64] }
        }
    }

    impl Bus<usize, u32, u8> for MockBus {
        type Error = ();

        fn write(&mut self, addr: usize, data: u32, strb: u8) -> Result<(), ()> {
            for lane in 0..4 {
                if ((strb >> lane) & 1) == 1 {
                    self.mem[addr + lane] = ((data >> (lane * 8)) & 0xFF) as u8;
                }
            }
            Ok(())
        }

        fn read(&mut self, addr: usize) -> Result<u32, ()> {
            let mut data = 0u32;
            for lane in 0..4 {
                data |= (self.mem[addr + lane] as u32) << (lane * 8);
            }
            Ok(data)
        }
    }

    #[test]
    fn clone_shares_same_bus() {
        let accessor: SharedBusAccessor<MockBus, usize, u32, u8, LittleEndian> =
            SharedBusAccessor::new(MockBus::default());

        let clone = accessor.clone();
        accessor.write_u32(0, 0xDEAD_BEEF).unwrap();
        assert_eq!(clone.read_u32(0).unwrap(), 0xDEAD_BEEF);
    }

    #[test]
    fn subclone_offset_and_bounds() {
        let root: SharedBusAccessor<MockBus, usize, u32, u8, LittleEndian> =
            SharedBusAccessor::new_with_range(MockBus::default(), 0, 64);

        // オフセット 16 バイト、サイズ 16 バイトのサブリージョン
        let sub = root.subclone(16, 16);
        assert_eq!(sub.base(), 16);
        assert_eq!(sub.size(), 16);

        // sub 経由で書き込み
        sub.write_u32(0, 0x1234_5678).unwrap();

        // root から offset=16 で同じ値が見える
        assert_eq!(root.read_u32(16).unwrap(), 0x1234_5678);

        // 境界外アクセスはエラー
        assert_eq!(
            sub.write_u32(13, 0).unwrap_err(),
            BusAccessorError::OutOfBounds
        );
    }

    #[test]
    fn subclone_nested() {
        let root: SharedBusAccessor<MockBus, usize, u32, u8, LittleEndian> =
            SharedBusAccessor::new(MockBus::default());

        let sub1 = root.subclone(8, 32);  // base=8, size=32
        let sub2 = sub1.subclone(4, 8);   // base=12, size=8

        assert_eq!(sub2.base(), 12);
        assert_eq!(sub2.size(), 8);

        sub2.write_u32(0, 0xCAFE_BABE).unwrap();
        assert_eq!(root.read_u32(12).unwrap(), 0xCAFE_BABE);
    }

    #[test]
    fn big_endian_subclone() {
        let root: SharedBusAccessor<MockBus, usize, u32, u8, LittleEndian> =
            SharedBusAccessor::new(MockBus::default());

        // エンディアンを BE に変えたサブクローン
        let be_sub = root.subclone_::<BigEndian>(0, 0);
        be_write_read_roundtrip(be_sub);
    }

    fn be_write_read_roundtrip(acc: SharedBusAccessor<MockBus, usize, u32, u8, BigEndian>) {
        acc.write_u64(0, 0x0011_2233_4455_6677).unwrap();
        assert_eq!(acc.read_u64(0).unwrap(), 0x0011_2233_4455_6677);
    }
}
