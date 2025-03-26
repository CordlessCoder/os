use core::{arch::x86_64::_rdtsc, time::Duration};

use crate::prelude::*;

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) -> ! {
    use crate::prelude::*;
    use crate::qemu::{QemuExitCode, exit_qemu};

    serial_println!("Running {} tests", tests.len());
    for &test in tests {
        let start = unsafe { _rdtsc() };
        test.run();
        let end = unsafe { _rdtsc() };
        let took = end - start;
        let approx = Duration::from_secs_f64(took as f64 / 5_000_000_000.);
        serial_println!(
            "Test took {took} cycles, approx {approx:?} assuming stable 5GHz(very inaccurate)."
        )
    }
    exit_qemu(QemuExitCode::Success);
}
