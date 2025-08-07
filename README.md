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
- **Interrupt handling**: Simplifies UIO interrupt management in Linux, including polling and waiting for IRQs.
- **DMA buffer support**: Seamless integration with `u-dma-buf` for high-performance data transfer.

This library is ideal for embedded systems developers and those working with custom hardware requiring efficient memory-mapped I/O access.

---

## MMIO (Memory Mapped I/O)

Direct MMIO access for bare-metal or Linux environments. Useful for register operations on custom hardware.

```rust
// Example: Bare-metal MMIO access
    type RegisterWordSize = u64;
    let mmio_acc = MmioAccessor::<RegisterWordSize>::new(0xffff0000, 0x10000);
    mmio_acc.write_mem_u8(0x00, 0x12);        // addr : 0xffff0000
    mmio_acc.write_mem_u16(0x02, 0x1234);     // addr : 0xffff0002
    mmio_acc.write_reg_u32(0x10, 0x12345678); // addr : 0xffff0080 <= 0x10 * size_of<RegisterWordSize>()
    mmio_acc.read_reg_u32(0x10);              // addr : 0xffff0080 <= 0x10 * size_of<RegisterWordSize>()
```

---

## UIO (Userspace I/O)

Linux UIO provides user-space access to device memory and interrupts via `/dev/uio*`. This library makes interrupt handling and register access easy.

```rust
// Example: UIO access and interrupt handling
    type RegisterWordSize = usize;
    let uio_num = 1;  // ex.) /dev/uio1
    let uio_acc = UioAccessor::<RegisterWordSize>::new(uio_num).unwrap();
    uio_acc.set_irq_enable(true).unwrap(); // Enable IRQ
    uio_acc.write_reg_u32(0x00, 0x1).unwrap();
    let irq_count = uio_acc.wait_irq().unwrap(); // Wait for IRQ
    println!("IRQ count: {}", irq_count);

// Polling for IRQ (non-blocking)
    if uio_acc.peek_irq(100).unwrap() {
        println!("IRQ detected!");
    }
    // Or, get IRQ count if available (returns None on timeout)
    if let Some(count) = uio_acc.poll_irq(100).unwrap() {
        println!("IRQ count: {}", count);
    }
```

You can also open it by specifying a name obtained from /sys/class/uio:

```rust
    let uio_acc = UioAccessor::<u32>::new_with_name("uio-sample").unwrap();
```

---

## u-dma-buf

Efficient DMA buffer management for high-speed data transfer. Requires the [u-dma-buf](https://github.com/ikwzm/udmabuf/) kernel module.

```rust
use jelly_mem_access::MemAccess;
    let udmabuf_num = 4;  // ex.) /dev/udmabuf4
    let udmabuf_acc = UdmabufAccessor::<usize>::new("udmabuf4", false).unwrap();
    println!("udmabuf4 phys addr : 0x{:x}", udmabuf_acc.phys_addr()); // MemAccess trait required
    println!("udmabuf4 size      : 0x{:x}", udmabuf_acc.size());      // MemAccess trait required
    unsafe {
        udmabuf_acc.write_mem_u32(0x00, 0x1234); // DMA buffer write (unsafe for direct memory access)
    }
```

---

## /dev/mem

Accessing `/dev/mem` allows direct memory-mapped I/O to physical addresses. **Note: root privileges are required. Use with caution.**

```rust
    let mem_acc = MmapAccessor::<usize>::new("/dev/mem", 0xa0000000, 0x1000).unwrap();
    mem_acc.write_reg_u32(0x10, 0x12345678).unwrap();
    let value = mem_acc.read_reg_u32(0x10).unwrap();
    println!("Value at register 0x10: 0x{:x}", value);
```

---

## subclone (Partial Region Cloning)

The `subclone` method allows you to clone a part of the memory region as a new accessor. This is useful for accessing a specific sub-region or for register-level operations with different types.

```rust
// Example: Clone a sub-region as u8 type
let mmio_acc = MmioAccessor::<u32>::new(0xffff0000, 0x10000);
let sub_acc = mmio_acc.subclone8(0x100, 0x10); // Access 0xffff0100 for 0x10 bytes as u8
sub_acc.write_mem_u8(0x00, 0xAA);
let val = sub_acc.read_mem_u8(0x00);
println!("Value in subclone region: 0x{:x}", val);
```

---

## License

MIT License
