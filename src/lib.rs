#[cfg(feature = "app")]
mod app;

#[cfg(feature = "app")]
pub use app::*;

mod protocol;

#[cfg(feature = "sender")]
pub use protocol::*;

/// 图形顶点
#[derive(Clone, Copy, Default)]
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

#[derive(Clone, Copy, PartialEq)]
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

#[test]
fn assert_size() {
    assert_eq!(16, std::mem::size_of::<Vertex>());
}
