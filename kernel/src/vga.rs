mod buffer;
pub mod color;
pub mod macros;
mod repr;
use core::{fmt::Write, ptr::NonNull};

pub use buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, FrameBuffer};
pub use repr::*;
use spinlock::{DisableInterrupts, SpinLock};
use x86_64::instructions::port::Port;

pub struct Writer {
    column: usize,
    pub color: ColorCode,
    pub buf: FrameBuffer,
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.bytes()
            .map(|b| match b {
                0x20..=0x7e | b'\n' => b,
                _ => 0xfe,
            })
            .for_each(|b| self.write_byte(b));
        self.move_cursor(BUFFER_HEIGHT as u8 - 1, self.column as u8);
        Ok(())
    }
}

/// The global VGA Text Mode output.
///
/// Implicitly locked by [print!](crate::print!)/[println!](crate::println!).
pub static VGA_OUT: SpinLock<Writer, DisableInterrupts> =
    SpinLock::disable_interrupts(Writer::new(unsafe {
        FrameBuffer::new(NonNull::new_unchecked(0xb8000 as *mut _))
    }));

/// Initialize the VGA Text Mode output
pub fn init() {
    VGA_OUT.lock().fill_screen(b' ');
}

impl Writer {
    pub const fn new(buf: FrameBuffer) -> Self {
        Self {
            column: 0,
            color: ColorCode::WHITE,
            buf,
        }
    }
    pub fn move_cursor(&mut self, row: u8, col: u8) {
        let pos = row as u16 * BUFFER_WIDTH as u16 + col as u16;
        unsafe {
            let mut control = Port::new(0x3D4);
            let mut data = Port::new(0x3d5);
            control.write(0x0Fu8);
            data.write(pos as u8);
            control.write(0x0Eu8);
            data.write((pos >> 8) as u8);
        }
    }
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.new_line()
                }
                const ROW: usize = BUFFER_HEIGHT - 1;
                self.buf.write_one(ROW, self.column, self.color, byte);
                self.column += 1;
            }
        }
    }
    fn with_current_color(&self, ascii: u8) -> ScreenChar {
        ScreenChar {
            ascii,
            color: self.color,
        }
    }
    pub fn new_line(&mut self) {
        let clear = self.with_current_color(b' ');
        self.buf.map_framebuffer(|mut buf| {
            for row in 1..BUFFER_HEIGHT {
                // self.buf.copy_row(row, row - 1);
                buf[row - 1] = buf[row];
            }
            buf[BUFFER_HEIGHT - 1] = [clear; BUFFER_WIDTH];
            buf
        });
        self.column = 0;
    }
    pub fn reset_column(&mut self) {
        self.column = 0;
    }
    pub fn fill_row(&mut self, row: usize, ascii: u8) {
        self.buf.splat_row(row, self.with_current_color(ascii));
    }
    pub fn fill_screen(&mut self, ascii: u8) {
        for row in 0..BUFFER_HEIGHT {
            self.buf.splat_row(row, self.with_current_color(ascii));
        }
    }
}
#[test_case]
fn test_println_many() {
    use crate::prelude::*;
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use crate::prelude::*;
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut writer = VGA_OUT.lock();
        for _ in 0..80 {
            let s = "Some test string that fits on a single line";
            _ = writeln!(writer, "\n{}", s);
            let row = writer.buf.read_row(BUFFER_HEIGHT - 2);
            for (i, c) in s.chars().enumerate() {
                assert_eq!(char::from(row[i].ascii), c);
            }
        }
    })
}
