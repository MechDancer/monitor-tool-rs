﻿#![deny(warnings)]

#[cfg(feature = "app")]
mod app;

#[cfg(feature = "app")]
pub use app::*;

mod protocol;

#[cfg(feature = "client")]
pub use protocol::*;

#[cfg(feature = "client")]
pub extern crate palette;

#[macro_export]
macro_rules! vertex {
    ($level:expr; $x:expr, $y:expr; $shape:ident, $extra:expr; $tie:expr) => {
        $crate::Vertex {
            x: $x as f32,
            y: $y as f32,
            level: $level,
            alpha: $tie,
            _zero: 0,
            shape: $crate::Shape::$shape,
            extra: $extra,
        }
    };
    ($level:expr; $x:expr, $y:expr; $alpha:expr) => {
        vertex!($level; $x, $y; Arrow, f32::NAN; $alpha)
    };
    ($level:expr; $x:expr, $y:expr => $theta:expr; $alpha:expr) => {
        vertex!($level; $x, $y; Arrow, $theta; $alpha)
    };
}

#[macro_export]
macro_rules! rgba {
    ($named:ident; $alpha:expr) => {
        $crate::palette::Srgba {
            color: $crate::palette::named::$named.into_format(),
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

impl Default for Shape {
    fn default() -> Self {
        Self::Arrow
    }
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Arrow => write!(f, "arrow "),
            Self::Circle => write!(f, "circle"),
        }
    }
}

#[test]
fn assert_size() {
    assert_eq!(16, std::mem::size_of::<Vertex>());
}
