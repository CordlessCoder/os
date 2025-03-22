use super::InterruptIndex;
use crate::prelude::{vga_color::*, *};
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!(
        fgcolor = LightRed,
        "EXCEPTION: BREAKPOINT\n{:#?}", stack_frame
    );
}
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        super::PICS
            .lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}
