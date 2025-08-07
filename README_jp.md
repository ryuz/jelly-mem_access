# メモリマップドI/Oアクセスライブラリ

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][license-badge]][license-url]

[crates-badge]: https://img.shields.io/crates/v/jelly-mem_access.svg
[crates-url]: https://crates.io/crates/jelly-mem_access
[license-badge]: https://img.shields.io/github/license/ryuz/jelly
[license-url]: https://github.com/ryuz/jelly/blob/master/license.txt

## 概要

`jelly-mem_access`ライブラリは、ベアメタル環境とLinux環境の両方でメモリマップドI/O（MMIO）にアクセスするための統一インターフェースを提供します。レジスタアクセスやメモリ操作を簡素化し、以下のような幅広い用途に対応しています：

- **ベアメタルプログラミング**：`no_std`対応で直接MMIOアクセス。
- **Linux UIO（Userspace I/O）**：ユーザー空間での割り込み処理やメモリアクセスを簡単に実現。
- **u-dma-buf**：[u-dma-buf](https://github.com/ikwzm/udmabuf/)カーネルモジュールを利用した効率的なDMAバッファ管理。

### 主な特徴
- **クロスプラットフォーム対応**：ベアメタルとLinuxの両方で動作。
- **柔軟なメモリアクセス**：u8, u16, u32, u64など様々なサイズのレジスタ読み書きAPIを提供。
- **割り込み処理**：LinuxでのUIO割り込み管理（ポーリング・待機）が簡単。
- **DMAバッファ対応**：`u-dma-buf`とのシームレスな連携で高速データ転送。

組込みシステム開発者や、効率的なメモリマップドI/Oアクセスが必要なカスタムハードウェア向けに最適です。

---

## MMIO（メモリマップドI/O）

ベアメタルやLinux環境で直接MMIOアクセスが可能です。カスタムハードウェアのレジスタ操作に便利です。

```rust
// ベアメタルMMIOアクセス例
    type RegisterWordSize = u64;
    let mmio_acc = MmioAccessor::<RegisterWordSize>::new(0xffff0000, 0x10000);
    mmio_acc.write_mem_u8(0x00, 0x12);        // addr : 0xffff0000
    mmio_acc.write_mem_u16(0x02, 0x1234);     // addr : 0xffff0002
    mmio_acc.write_reg_u32(0x10, 0x12345678); // addr : 0xffff0080 <= 0x10 * size_of<RegisterWordSize>()
    mmio_acc.read_reg_u32(0x10);              // addr : 0xffff0080 <= 0x10 * size_of<RegisterWordSize>()
```

---

## UIO（Userspace I/O）

Linux UIOは、`/dev/uio*`経由でデバイスメモリや割り込みにユーザー空間からアクセスできます。本ライブラリで割り込み処理やレジスタアクセスが簡単になります。

```rust
// UIOアクセスと割り込み処理例
    type RegisterWordSize = usize;
    let uio_num = 1;  // 例：/dev/uio1
    let uio_acc = UioAccessor::<RegisterWordSize>::new(uio_num).unwrap();
    uio_acc.set_irq_enable(true).unwrap(); // IRQ有効化
    uio_acc.write_reg_u32(0x00, 0x1).unwrap();
    let irq_count = uio_acc.wait_irq().unwrap(); // IRQ待機
    println!("IRQ count: {}", irq_count);

// IRQのポーリング（非ブロッキング）
    if uio_acc.peek_irq(100).unwrap() {
        println!("IRQ検出！");
    }
    // IRQカウント取得（タイムアウト時はNone）
    if let Some(count) = uio_acc.poll_irq(100).unwrap() {
        println!("IRQ count: {}", count);
    }
```

`/sys/class/uio`から取得したデバイス名でオープンすることも可能です：

```rust
    let uio_acc = UioAccessor::<u32>::new_with_name("uio-sample").unwrap();
```

---

## u-dma-buf

高速データ転送のためのDMAバッファ管理。カーネルモジュール[u-dma-buf](https://github.com/ikwzm/udmabuf/)が必要です。

```rust
use jelly_mem_access::MemAccess;
    let udmabuf_num = 4;  // 例：/dev/udmabuf4
    let udmabuf_acc = UdmabufAccessor::<usize>::new("udmabuf4", false).unwrap();
    println!("udmabuf4 phys addr : 0x{:x}", udmabuf_acc.phys_addr()); // MemAccessトレイト必須
    println!("udmabuf4 size      : 0x{:x}", udmabuf_acc.size());      // MemAccessトレイト必須
    unsafe {
        udmabuf_acc.write_mem_u32(0x00, 0x1234); // DMAバッファ書き込み（unsafe：直接メモリアクセス）
    }
```

---

## /dev/mem

`/dev/mem`を使うことで物理アドレスへの直接メモリマップドI/Oが可能です。**注意：root権限が必要です。利用時は十分注意してください。**

```rust
    let mem_acc = MmapAccessor::<usize>::new("/dev/mem", 0xa0000000, 0x1000).unwrap();
    mem_acc.write_reg_u32(0x10, 0x12345678).unwrap();
    let value = mem_acc.read_reg_u32(0x10).unwrap();
    println!("Value at register 0x10: 0x{:x}", value);
```

---

## subclone（部分領域の複製）

`subclone`メソッドを使うことで、メモリ領域の一部を新しいアクセサとして複製できます。部分的な領域アクセスや、レジスタ単位の操作に便利です。

```rust
// 例：領域の一部をu8型で複製
let mmio_acc = MmioAccessor::<u32>::new(0xffff0000, 0x10000);
let sub_acc = mmio_acc.subclone8(0x100, 0x10); // 0xffff0100から0x10バイト分をu8型でアクセス
sub_acc.write_mem_u8(0x00, 0xAA);
let val = sub_acc.read_mem_u8(0x00);
println!("subclone領域の値: 0x{:x}", val);
```

---

## ライセンス

MIT License
