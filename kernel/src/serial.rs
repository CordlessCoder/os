use core::cell::UnsafeCell;
use spinlock::LazyLock;
use uart_16550::SerialPort;

pub static SERIAL1: LazyLock<SerialPort> = LazyLock::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    UnsafeCell::new(serial_port)
});

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    let mut serial = SERIAL1.lock();
    serial.write_fmt(args).expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    (interrupts_disabled $($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
    ($($arg:tt)*) => {
        ::x86_64::instructions::interrupts::without_interrupts(|| {
            $crate::serial::_print(format_args!($($arg)*));
        })
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    (interrupts_disabled) => (interrupts_disabled $crate::serial_print!("\n"));
    (interrupts_disabled $fmt:expr) => ($crate::serial_print!(interrupts_disabled concat!($fmt, "\n")));
    (interrupts_disabled $fmt:expr, $($arg:tt)*) => {
        $crate::serial_print!(interrupts_disabled concat!($fmt, "\n"), $($arg)*);
    };
    ($($arg:tt)*) => {
        ::x86_64::instructions::interrupts::without_interrupts(|| {
            $crate::serial_println!(interrupts_disabled $($arg)*);
        })
    }
}
