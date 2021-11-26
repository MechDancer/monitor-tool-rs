#[cfg(feature = "app")]
mod app;

use std::fmt::Display;

#[cfg(feature = "app")]
pub use app::*;

mod protocol;

#[cfg(feature = "sender")]
pub use protocol::*;

#[macro_export]
macro_rules! vertex {
    ($level:expr; $x:expr, $y:expr; $shape:ident, $extra:expr; $tie:expr) => {
        monitor_tool::Vertex {
            x: $x,
            y: $y,
            level: $level,
            alpha: $tie,
            _zero: 0,
            shape: monitor_tool::Shape::$shape,
            extra: $extra,
        }
    };
    ($level:expr; $x:expr, $y:expr; $alpha:expr) => {
        vertex!($level; $x, $y; Arrow, f32::NAN; $alpha)
    };
}

#[macro_export]
macro_rules! rgba {
    ($named:ident; $alpha:expr) => {
        palette::Srgba {
            color: palette::named::$named.into_format(),
            alpha: $alpha,
        }
    };
}

/// 图形顶点
#[derive(Clone, Copy, PartialEq, Default, Debug)]
#[repr(C)]
pub struct Vertex {
    pub x: f32,       // 位置 x
    pub y: f32,       // 位置 y
    pub level: u8,    // 等级
    pub alpha: u8,    // 连线透明度
    pub _zero: u8,    // 占位
    pub shape: Shape, // 补充数据类型
    pub extra: f32,   // 补充数据
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shape {
    Arrow,
    Circle,
}

impl Vertex {
    #[cfg(feature = "app")]
    #[inline]
    pub fn pos(&self) -> iced::Point {
        iced::Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self::Arrow
    }
}

impl Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::Arrow => write!(f, "arrow "),
            &Self::Circle => write!(f, "circle"),
        }
    }
}

#[test]
fn assert_size() {
    assert_eq!(16, std::mem::size_of::<Vertex>());
}
