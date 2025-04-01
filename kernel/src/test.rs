use crate::{clock::Instant, prelude::*};

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
        let start = Instant::now();
        test.run();
        let took = start.elapsed();
        serial_println!("Test took {took:?}")
    }
    exit_qemu(QemuExitCode::Success);
}
