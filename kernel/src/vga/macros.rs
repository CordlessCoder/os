use crate::vga::VGA_OUT;
use core::fmt;

#[macro_export]
macro_rules! print {
    (interrupts_disabled bgcolor = $bg:expr, $($arg:tt)*) => {
        VGA_OUT.lock().color.set_bg($bg);
        $crate::print!(interrupts_disabled $($arg)*);
    };
    (interrupts_disabled fgcolor = $fg:expr, $($arg:tt)*) => {
        VGA_OUT.lock().color.set_fg($fg);
        $crate::print!(interrupts_disabled $($arg)*);
    };
    (interrupts_disabled $($arg:tt)*) => ($crate::vga::macros::_print(format_args!($($arg)*)));
    ($($arg:tt)*) => {
        ::x86_64::instructions::interrupts::without_interrupts(|| {
            $crate::print!(interrupts_disabled $($arg)*);
        })
    }
}

#[macro_export]
macro_rules! println {
    (interrupts_disabled bgcolor = $bg:expr, $($arg:tt)*) => {
        VGA_OUT.lock().color.set_bg($bg);
        $crate::println!(interrupts_disabled $($arg)*)
    };
    (interrupts_disabled fgcolor = $fg:expr, $($arg:tt)*) => {
        VGA_OUT.lock().color.set_fg($fg);
        $crate::println!(interrupts_disabled $($arg)*);
    };
    (interrupts_disabled $($arg:tt)*) => {
        $crate::print!(interrupts_disabled "{}\n", format_args!($($arg)*))
    };
    (interrupts_disabled) => ($crate::print!(interrupts_disabled "\n"));
    ($($arg:tt)*) => {
        ::x86_64::instructions::interrupts::without_interrupts(|| {
            $crate::println!(interrupts_disabled $($arg)*);
        })
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    VGA_OUT.lock().write_fmt(args).unwrap();
}
