use super::InterruptIndex;
use crate::{
    interrupts::PICS,
    prelude::{vga_color::*, *},
};
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use spinlock::{LazyStatic, SpinLock};
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    println!(
        fgcolor = LightRed,
        "EXCEPTION: BREAKPOINT\n{:#?}", stack_frame
    );
}
pub extern "x86-interrupt" fn double_fault(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn timer_interrupt(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

pub extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: InterruptStackFrame) {
    static KEYBOARD: LazyStatic<SpinLock<Keyboard<layouts::Us104Key, ScancodeSet1>>> =
        LazyStatic::new(|| {
            SpinLock::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore,
            ))
        });
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let mut keyboard = KEYBOARD.lock();

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
