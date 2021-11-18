use async_std::{
    net::UdpSocket,
    sync::{Arc, Mutex},
    task,
};
use iced::{
    canvas::{event, Cache, Cursor, Event, Geometry, Program},
    futures::stream::{repeat_with, BoxStream},
    mouse, Color, Point, Rectangle, Size,
};
use iced_futures::subscription::Recipe;
use std::time::Instant;

mod figure;
mod figure_canvas;
mod protocol;

use figure::Figure;
use figure_canvas::*;
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

#[derive(Clone)]
pub struct FigureProgram(Arc<Mutex<FigureCanvas>>);

pub struct UdpReceiver(u16);

#[derive(Debug)]
pub enum Message {
    MessageReceived(Instant, Vec<u8>),
    ViewUpdated,
}

struct FigureCanvas {
    update_time: Instant,
    border_cache: Cache,
    figure: Figure,
}

impl Vertex {
    #[inline]
    pub fn pos(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl FigureProgram {
    #[inline]
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(FigureCanvas {
            update_time: Instant::now(),
            border_cache: Default::default(),
            figure: Default::default(),
        })))
    }

    #[inline]
    pub fn receive(&self, time: Instant, buf: &[u8]) {
        decode(&mut task::block_on(self.0.lock()).figure, time, buf);
    }
}

impl Program<Message> for FigureProgram {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        let pos = if let Some(position) = cursor.position_in(&bounds) {
            position
        } else {
            return (event::Status::Ignored, None);
        };
        let bounds = bounds.size();

        use mouse::{Event::*, ScrollDelta};
        match event {
            event::Event::Mouse(mouse_event) => match mouse_event {
                WheelScrolled {
                    delta: ScrollDelta::Lines { x: _, y } | ScrollDelta::Pixels { x: _, y },
                } => {
                    task::block_on(async {
                        let figure = &mut self.0.lock().await.figure;
                        figure.auto_view = false;
                        figure.zoom(y, pos, bounds);
                    });
                    (event::Status::Captured, Some(Message::ViewUpdated))
                }
                _ => (event::Status::Ignored, None),
            },
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let size = bounds.size();
        let (rectangle, mut geometries) = task::block_on(self.0.lock()).draw(size);
        if let Cursor::Available(p) = cursor {
            if is_available(size, p) {
                geometries.push(mark_cross(size, p, rectangle));
            }
        }
        geometries
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if let Cursor::Available(p) = cursor {
            if is_available(bounds.size(), p) {
                return mouse::Interaction::Crosshair;
            }
        }
        mouse::Interaction::default()
    }
}

impl FigureCanvas {
    fn draw(&mut self, size: Size) -> (Rectangle, Vec<Geometry>) {
        let time = Instant::now();
        // 绘制数据
        let mut result = self.figure.draw(size, available_size(size));
        // 绘制边框
        result.1.push(
            self.border_cache
                .draw(size, |frame| border(frame, Color::BLACK)),
        );
        // 计时
        println!(
            "period = {:?}, delay = {:?}",
            time - std::mem::replace(&mut self.update_time, time),
            Instant::now() - time,
        );
        result
    }
}

impl UdpReceiver {
    pub const fn new(port: u16) -> Self {
        Self(port)
    }
}

impl<H, E> Recipe<H, E> for UdpReceiver
where
    H: std::hash::Hasher,
{
    type Output = Message;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
        self.0.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, E>) -> BoxStream<'static, Self::Output> {
        let socket =
            Arc::new(task::block_on(UdpSocket::bind(format!("0.0.0.0:{}", self.0))).unwrap());
        let mut buf = [0u8; 65536];
        Box::pin(repeat_with(move || {
            let socket = socket.clone();
            let (len, _) = task::block_on(socket.recv_from(&mut buf)).unwrap();
            Message::MessageReceived(Instant::now(), buf[..len].to_vec())
        }))
    }
}
