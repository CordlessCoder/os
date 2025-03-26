use core::panic::PanicInfo;
use core::sync::atomic::AtomicBool;

static QEMU_TEST_PANIC: AtomicBool = AtomicBool::new(false);

pub fn set_qemu_test_panic() {
    QEMU_TEST_PANIC.store(true, core::sync::atomic::Ordering::Release);
}

/// # Safety
/// The caller must ensure the VGA_OUT writer is not accessed by other threads when this function
/// is called
/// # Safety
/// The caller must ensure the SERIAL1 writer is not accessed by other threads when this function
/// is called
#[panic_handler]
pub unsafe fn panic(info: &PanicInfo) -> ! {
    use crate::prelude::{vga_color::*, *};
    if QEMU_TEST_PANIC.load(core::sync::atomic::Ordering::Acquire) {
        use crate::prelude::*;
        use crate::qemu::{QemuExitCode, exit_qemu};
        use core::fmt::Write;

        // SAFETY: The panic may have happened inside a fmt::Display implementation,
        // which would leave the SpinLock locked forever.
        let out = unsafe { &mut *SERIAL1.get_inner_mut() };
        _ = writeln!(out, "[failed]\n");
        _ = writeln!(out, "Error: {}\n", info);
        exit_qemu(QemuExitCode::Failed);
    }

    // SAFETY: The panic may have happened inside a fmt::Display implementation,
    // which would leave the SpinLock locked forever.
    println!(fgcolor = LightRed, bgcolor = Black, "{info}");
    crate::hlt_loop()
}
