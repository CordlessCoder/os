use core::panic::PanicInfo;

/// # Safety
/// The caller must ensure the VGA_OUT writer is not accessed by other threads when this function
/// is called
#[panic_handler]
#[cfg(not(test))]
pub unsafe fn panic(info: &PanicInfo) -> ! {
    use crate::prelude::{vga_color::*, *};

    // SAFETY: The panic may have happened inside a fmt::Display implementation,
    // which would leave the SpinLock locked forever.
    println!(fgcolor = LightRed, bgcolor = Black, "{info}");
    loop {}
}
/// # Safety
/// The caller must ensure the SERIAL1 writer is not accessed by other threads when this function
/// is called
#[panic_handler]
#[cfg(test)]
pub unsafe fn panic(info: &PanicInfo) -> ! {
    use crate::prelude::*;
    use crate::qemu::{QemuExitCode, exit_qemu};
    use core::fmt::Write;

    // SAFETY: The panic may have happened inside a fmt::Display implementation,
    // which would leave the SpinLock locked forever.
    let out = unsafe {
        let out = &mut *SERIAL1.get_inner_mut();
        let out = core::cell::LazyCell::force(out);
        &mut *out.get()
    };
    _ = writeln!(out, "[failed]\n");
    _ = writeln!(out, "Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}
