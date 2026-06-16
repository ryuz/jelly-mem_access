#![allow(dead_code)]

use core::marker::PhantomData;
use std::sync::{Arc, Mutex};

use super::bus_accessor::{Bus, BusAccessorError, BusAddress, BusValue, BusWord, Endianness};

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
