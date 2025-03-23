#![no_std]
#![feature(custom_test_frameworks, abi_x86_interrupt)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_main]
use bootloader::{BootInfo, entry_point};
use kernel::{
    prelude::*,
    qemu::{QemuExitCode, exit_qemu},
};
use spinlock::LazyStatic;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

entry_point!(main);
fn main(_: &'static BootInfo) -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    kernel::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

static TEST_IDT: LazyStatic<InterruptDescriptorTable> = LazyStatic::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
        idt.double_fault
            .set_handler_fn(test_double_fault_handler)
            .set_stack_index(kernel::gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt
});

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    let mut val = 0;
    let mut val = volatile::VolatileRef::from_mut_ref(&mut val);
    val.as_mut_ptr().write(0);
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
