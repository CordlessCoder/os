use super::color::{BgColor, Color, FgColor, LightColor};

/// A byte-encoded VGA Text Mode Color.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const WHITE: Self = ColorCode((Color::Black as u8) << 4 | LightColor::White as u8 | 0x8);
    pub fn new(foreground: impl FgColor, background: impl BgColor) -> Self {
        ColorCode(background.bg_repr() << 4 | foreground.fg_repr())
    }
    pub fn set_fg(&mut self, fg: impl FgColor) {
        let bg = self.0 >> 4;
        *self = ColorCode(bg << 4 | fg.fg_repr())
    }
    pub fn set_bg(&mut self, bg: impl BgColor) {
        let fg = self.0 & 0b1111;
        *self = ColorCode(bg.bg_repr() << 4 | fg);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii: u8,
    pub color: ColorCode,
}
