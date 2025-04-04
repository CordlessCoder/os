use crate::vga::VGA_OUT;
use core::fmt;

/// Print to the VGA Console.
///
/// Allows for setting the foreground and background color by providing `fgcolor = COLOR` and
/// `bgcolor = COLOR` arguments.
/// Example: `print!(fgcolor = Red, "I am red!")`
#[macro_export]
macro_rules! print {
    (bgcolor = $bg:expr, $($arg:tt)*) => {{
        VGA_OUT.lock().color.set_bg($bg);
        $crate::print!($($arg)*)
    }};
    (fgcolor = $fg:expr, $($arg:tt)*) => {{
        VGA_OUT.lock().color.set_fg($fg);
        $crate::print!($($arg)*)
    }};
    ($($arg:tt)*) => ($crate::vga::macros::_print(format_args!($($arg)*)));
}

/// Print to the VGA Console.
///
/// Allows for setting the foreground and background color by providing `fgcolor = COLOR` and
/// `bgcolor = COLOR` arguments.
/// Example: `println!(fgcolor = Red, "I am red!")`
#[macro_export]
macro_rules! println {
    (bgcolor = $bg:expr, $($arg:tt)*) => {{
        VGA_OUT.lock().color.set_bg($bg);
        $crate::println!($($arg)*)
    }};
    (fgcolor = $fg:expr, $($arg:tt)*) => {{
        VGA_OUT.lock().color.set_fg($fg);
        $crate::println!($($arg)*)
    }};
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    VGA_OUT.lock().write_fmt(args).unwrap();
}
