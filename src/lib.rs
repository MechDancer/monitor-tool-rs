#[cfg(feature = "app")]
mod figure_program;

#[cfg(feature = "app")]
mod cache_builder;

#[cfg(feature = "app")]
mod udp_receiver;

#[cfg(feature = "app")]
pub use app::*;

#[cfg(feature = "app")]
mod app {
    use super::*;
    pub use cache_builder::spawn_background as spawn_draw;
    pub(crate) use figure_program::Figure;
    pub use figure_program::{CacheComplete, FigureEvent, FigureProgram};
    pub use udp_receiver::spawn_background as spawn_receive;
}

mod protocol;

#[cfg(feature = "sender")]
pub use protocol::*;

/// 图形顶点
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Vertex {
    pub x: f32,     // 位置 x
    pub y: f32,     // 位置 y
    pub dir: f32,   // 方向 θ
    pub level: u8,  // 等级
    pub tie: bool,  // 是否与上一个点相连
    pub _zero: u16, // 占位
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
