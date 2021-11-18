#[cfg(feature = "app")]
mod decode;

#[cfg(feature = "app")]
pub(crate) use decode::decode;

#[cfg(feature = "sender")]
mod packet;

#[cfg(feature = "sender")]
pub use packet::*;

/// 颜色
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RGBA(pub u8, pub u8, pub u8, pub u8);

/// 摄像机设置
#[repr(C)]
pub struct Camera {
    x: f32,       // 中心点横坐标
    y: f32,       // 中心点纵坐标
    scale_x: f32, // TODO 应该改为宽
    scale_y: f32, // TODO 应该改为高
}

impl Default for Camera {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Camera {
    /// 不控制摄像机
    pub const DEFAULT: Self = Self {
        x: f32::NAN,
        y: f32::NAN,
        scale_x: f32::NAN,
        scale_y: f32::NAN,
    };

    /// 摄像机自动
    pub const AUTO: Self = Self {
        x: f32::NAN,
        y: f32::NAN,
        scale_x: 0.0,
        scale_y: 0.0,
    };
}

/// 图层是否显示
enum Visible {
    NothingToDo = 0,
    Visible = 0x55,
    Invisible = 0xaa,
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
