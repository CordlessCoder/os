use core::time::Duration;

use pic8259::ChainedPics;
use spinlock::{LazyStatic, SpinLock};
use x86_64::{
    instructions::{interrupts, port::Port},
    structures::idt::InterruptDescriptorTable,
};
mod handlers;

pub static PICS: SpinLock<ChainedPics> =
    SpinLock::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

static IDT: LazyStatic<InterruptDescriptorTable> = LazyStatic::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(handlers::breakpoint);
    unsafe {
        idt.double_fault
            .set_handler_fn(handlers::double_fault)
            .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX)
    };
    idt.page_fault.set_handler_fn(handlers::page_fault);
    idt[InterruptIndex::Timer as u8].set_handler_fn(handlers::timer_interrupt);
    idt[InterruptIndex::Keyboard as u8].set_handler_fn(handlers::keyboard_interrupt);
    idt
});

fn set_timer_freq(tick_every: Duration) {
    const OSCILLATOR_FREQ: f64 = 3579545. / 3.;
    let oscillator_interval = Duration::from_secs(1).div_f64(OSCILLATOR_FREQ);
    let counter = tick_every.div_duration_f32(oscillator_interval) as u16;
    interrupts::without_interrupts(|| unsafe {
        let mut command = Port::new(0x43);
        command.write(0b0011_0110u8);
        let mut timer = Port::new(0x40);
        timer.write((counter & 0xFF) as u8);
        timer.write((counter >> 8) as u8);
    })
}

pub fn init() {
    IDT.load();
    unsafe {
        set_timer_freq(Duration::from_millis(1));
        PICS.lock().initialize();
    };
    x86_64::instructions::interrupts::enable();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}
