use core::ptr::NonNull;
use volatile::VolatileRef;

use super::{ColorCode, ScreenChar};

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct FrameBuffer {
    chars: VolatileRef<'static, [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]>,
}

impl FrameBuffer {
    #[allow(private_interfaces)]
    pub const fn new(ptr: NonNull<[[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]>) -> Self {
        unsafe {
            Self {
                chars: VolatileRef::new_restricted(volatile::access::ReadWrite, ptr),
            }
        }
    }
    pub fn write_one(&mut self, row: usize, column: usize, color: ColorCode, ascii: u8) {
        unsafe {
            let row = self
                .chars
                .as_mut_ptr()
                .map(|mut p| NonNull::new_unchecked(&mut p.as_mut()[row]));
            let col = row.map(|mut p| NonNull::new_unchecked(&mut p.as_mut()[column]));
            col.write(ScreenChar { color, ascii });
        }
    }
    pub fn splat_row(&mut self, row: usize, data: ScreenChar) {
        self.set_row(row, [data; 80]);
    }
    pub fn map_framebuffer(
        &mut self,
        cb: impl FnOnce(
            [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
        ) -> [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    ) {
        self.chars.as_mut_ptr().update(cb);
    }
    pub fn copy_row(&mut self, from: usize, to: usize) {
        let data = self.read_row(from);
        self.set_row(to, data);
    }
    pub fn read_row(&mut self, row: usize) -> [ScreenChar; BUFFER_WIDTH] {
        unsafe {
            let row = self
                .chars
                .as_mut_ptr()
                .map(|mut p| NonNull::new_unchecked(&mut p.as_mut()[row]));
            row.read()
        }
    }
    pub fn set_row(&mut self, row: usize, data: [ScreenChar; BUFFER_WIDTH]) {
        unsafe {
            let row = self
                .chars
                .as_mut_ptr()
                .map(|mut p| NonNull::new_unchecked(&mut p.as_mut()[row]));
            row.write(data);
        }
    }
}
