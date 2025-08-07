# Memory Mapped I/O Access Library

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][license-badge]][license-url]

[crates-badge]: https://img.shields.io/crates/v/jelly-mem_access.svg
[crates-url]: https://crates.io/crates/jelly-mem_access
[license-badge]: https://img.shields.io/github/license/ryuz/jelly
[license-url]: https://github.com/ryuz/jelly/blob/master/license.txt


## Overview

The `jelly-mem_access` library provides a unified interface for accessing memory-mapped I/O (MMIO) in both bare-metal and Linux environments. It is designed to simplify register access and memory operations, supporting a wide range of use cases, including:

- **Bare-metal programming**: Direct MMIO access with `no_std` support.
- **Linux UIO (Userspace I/O)**: Easy integration with `/dev/uio` devices for user-space interrupt handling and memory access.
- **u-dma-buf**: Efficient DMA buffer management using the [u-dma-buf](https://github.com/ikwzm/udmabuf/) kernel module.

### Key Features
- **Cross-platform support**: Works in both bare-metal and Linux environments.
- **Flexible memory access**: Provides APIs for reading and writing registers of various sizes (e.g., u8, u16, u32, u64).
- **Interrupt handling**: Simplifies UIO interrupt management in Linux.
- **DMA buffer support**: Seamless integration with `u-dma-buf` for high-performance data transfer.

This library is ideal for embedded systems developers and those working with custom hardware requiring efficient memory-mapped I/O access.


## MMIO(Memory Mapped I/O)

MMIO access in bare-metal programming can be written as follows:

```rust
    type RegisterWordSize = u64;
    let mmio_acc = MmioAccessor::<RegisterWordSize>::new(0xffff0000, 0x10000);
    mmio_acc.write_mem_u8(0x00, 0x12);        // addr : 0xffff0000
    mmio_acc.write_mem_u16(0x02, 0x1234);     // addr : 0xffff0002
    mmio_acc.write_reg_u32(0x10, 0x12345678); // addr : 0xffff0080 <= 0x10 * size_of<RegisterWordSize>()
    mmio_acc.read_reg_u32(0x10);              // addr : 0xffff0080 <= 0x10 * size_of<RegisterWordSize>()
```


## UIO(Userspace I/O)

UIO access in Linux programming can be written as follows:

```rust
    type RegisterWordSize = usize;
    let uio_num = 1;  // ex.) /dev/uio1
    let uio_acc = UioAccessor::<RegisterWordSize>::new(uio_num).unwrap();
    uio_acc.set_irq_enable(true).unwrap();
    uio_acc.write_reg_u32(0x00, 0x1).unwrap();
    let irq_count = uio_acc.wait_irq().unwrap();
    println!("IRQ count: {}", irq_count);
```

You can also open it by specifying a name obtained from /sys/class/uio:

```rust
    let uio_acc = UioAccessor::<u32>::new_with_name("uio-sample").unwrap();
```

## u-dma-buf

[u-dma-buf](https://github.com/ikwzm/udmabuf/) access in Linux programming can be written as follows:

```rust
use jelly_mem_access::MemAccess;
    let udmabuf_num = 4;  // ex.) /dev/udmabuf4
    let udmabuf_acc = UdmabufAccessor::<usize>::new("udmabuf4", false).unwrap();
    println!("udmabuf4 phys addr : 0x{:x}", udmabuf_acc.phys_addr()); // MemAccessトレイトが必要
    println!("udmabuf4 size      : 0x{:x}", udmabuf_acc.size());      // MemAccessトレイトが必要
    unsafe {
        udmabuf_acc.write_mem_u32(0x00, 0x1234);
    }
```

## /dev/mem

Accessing `/dev/mem` for memory-mapped I/O can be written as follows:

```rust
    let mem_acc = MmapAccessor::<usize>::new("/dev/mem", 0xa0000000, 0x1000).unwrap();
    mem_acc.write_reg_u32(0x10, 0x12345678).unwrap();
    let value = mem_acc.read_reg_u32(0x10).unwrap();
    println!("Value at register 0x10: 0x{:x}", value);
```
