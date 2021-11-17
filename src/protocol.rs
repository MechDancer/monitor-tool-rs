mod packet;

pub use packet::*;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RGBA(u8, u8, u8, u8);

#[repr(C)]
pub struct Camera {
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
}

#[repr(C)]
enum Visible {
    NothingToDo = 0,
    Visible = 0x55,
    Invisible = 0xaa,
}

impl Default for Camera {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Camera {
    pub const DEFAULT: Self = Self {
        x: f32::NAN,
        y: f32::NAN,
        scale_x: f32::NAN,
        scale_y: f32::NAN,
    };

    pub const AUTO: Self = Self {
        x: f32::NAN,
        y: f32::NAN,
        scale_x: 0.0,
        scale_y: 0.0,
    };
}

impl Default for Visible {
    #[inline]
    fn default() -> Self {
        Self::NothingToDo
    }
}

impl From<Option<bool>> for Visible {
    #[inline]
    fn from(v: Option<bool>) -> Self {
        match v {
            None => Visible::NothingToDo,
            Some(true) => Visible::Visible,
            Some(false) => Visible::Invisible,
        }
    }
}
