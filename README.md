An x86-64 kernel written in Rust, largely following the [Writing an OS in Rust](https://os.phil-opp.com/) blog by [Philipp Oppermann](https://github.com/phil-opp).

# Features
- Cooperative multitasking implemented on top of Rust async.
- Global [millisecond-granular clock](https://github.com/CordlessCoder/os/blob/main/kernel/src/clock.rs)
    implemented via the [PIT](https://en.wikipedia.org/wiki/Programmable_interval_timer)[^INT].
- Convenient [VGA Text Mode handling utilities](https://github.com/CordlessCoder/os/blob/main/kernel/src/vga.rs)
    with [`println!`](https://github.com/CordlessCoder/os/blob/main/kernel/src/vga/macros.rs#L20) macro color integration.
- Custom [interrupt-aware spinlock-backed Mutex](https://github.com/CordlessCoder/os/blob/main/spinlock/src/lib.rs)
    and lock-free [LazyStatic implementation](https://github.com/CordlessCoder/os/blob/main/spinlock/src/lazystatic.rs).
- Global [freelist-backed heap allocator](https://github.com/CordlessCoder/os/blob/main/kernel/src/memory/freelist_alloc.rs)[^ALLOC].
- [Event-driven keyboard input](https://github.com/CordlessCoder/os/blob/main/kernel/src/task/keyboard.rs)[^INT].
- Support for [unit](https://github.com/CordlessCoder/os/blob/main/kernel/src/test.rs) and [integration](https://github.com/CordlessCoder/os/tree/main/kernel/tests) testing.
- BIOS support via the [`bootloader`](https://docs.rs/bootloader/0.9.31/bootloader/index.html) crate[^BOOTLOADER].

[^INT]: Timer and keyboard interrupts are handled via the [legacy PIC](https://wiki.osdev.org/8259_PIC).
    A transition to the [APIC](https://wiki.osdev.org/APIC) is in-progress.
[^BOOTLOADER]: The `bootimage` tool used is incompatible with versions of `bootloader` >= 0.10.
    Transitioning off of it will enable UEFI support and simplify the build process.
    This will also require switching from VGA Text Mode to VGA Graphics Mode.
[^ALLOC]: The freelist allocator is fairly simple and can be improved by using a block-allocator to allow for allocations <16 bytes in size,
    that falls back to the freelist allocator for larger allocations.

# Goal
The eventual goal of this project is to build an OS that can run Doom, but the current state of the kernel is very far from that.

### Current state
The kernel in its current state provides a sufficient interface for building Terminal UI applications and simple games,
however VGA Text Mode is very limiting and everything must run in kernel space.

As an example, I implemented a quick-and-dirty "shell"(glorified string literal matcher with rudimentary input handling) and two "apps".
They are snake and flappy bird.



### Future work
- VGA Graphics Mode
- Transitioning to `bootloader` version 0.11[^BOOTLOADER].
- Multithreading.
- Userspace and syscall support.
- Preemptive multitasking for userspace.
- Frame-aware global allocator.
- Read/Write/Seek syscalls.
- Memory mapping syscalls
- File system.
- Process management syscalls.
- Windowing.
- Exposing a graphics API to userspace.
- Porting a subset of libc.
- Doom.
