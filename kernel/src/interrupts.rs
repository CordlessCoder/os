use crate::prelude::{vga_color::*, *};
use spinlock::lazystatic::LazyStatic;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: LazyStatic<InterruptDescriptorTable> = LazyStatic::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt
});

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!(
        fgcolor = LightRed,
        "EXCEPTION: BREAKPOINT\n{:#?}", stack_frame
    );
}
