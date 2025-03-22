#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[repr(u8)]
pub enum LightColor {
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Blink(pub Color);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
}

pub trait FgColor {
    fn fg_repr(&self) -> u8;
}
pub trait BgColor {
    fn bg_repr(&self) -> u8;
}
impl FgColor for Color {
    fn fg_repr(&self) -> u8 {
        *self as u8
    }
}
impl BgColor for Color {
    fn bg_repr(&self) -> u8 {
        *self as u8
    }
}
impl FgColor for LightColor {
    fn fg_repr(&self) -> u8 {
        *self as u8
    }
}
impl BgColor for Blink {
    fn bg_repr(&self) -> u8 {
        self.0.bg_repr() | 0x8
    }
}
