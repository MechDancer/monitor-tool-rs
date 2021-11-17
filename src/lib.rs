use async_std::{
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    task,
};
use iced::{
    canvas::{event, Cache, Cursor, Event, Geometry, Program},
    futures::stream::{repeat_with, BoxStream},
    mouse, Color, Rectangle, Size,
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
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub dir: f32,
    pub level: u8,
    pub tie: bool,
}

#[derive(Clone)]
pub struct FigureProgram(Arc<Mutex<FigureCanvas>>);

pub struct Server(u16);

#[derive(Debug)]
pub enum Message {
    MessageReceived(Instant, SocketAddr, Vec<u8>),
    ViewUpdated,
}

struct FigureCanvas {
    update_time: Instant,
    border_cache: Cache,
    figure: Figure,
}

impl Default for Vertex {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Vertex {
    const DEFAULT: Self = Self {
        x: 0.0,
        y: 0.0,
        dir: 0.0,
        level: 0,
        tie: false,
    };
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
    pub fn receive(&self, src: SocketAddr, buf: &[u8]) {
        task::block_on(self.0.lock()).figure.receive(src, buf);
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
            event::Event::Mouse(mouse_event) => {
                match mouse_event {
                    WheelScrolled {
                        delta: ScrollDelta::Lines { x: _, y } | ScrollDelta::Pixels { x: _, y },
                    } => task::block_on(self.0.lock()).figure.zoom(y, pos, bounds),
                    _ => {}
                }
                (event::Status::Captured, Some(Message::ViewUpdated))
            }
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
        let now = Instant::now();
        // 绘制数据
        let mut result = self.figure.draw(size, available_size(size));
        // 绘制边框
        result.1.push(
            self.border_cache
                .draw(size, |frame| border(frame, Color::BLACK)),
        );
        // 计时
        {
            let last = std::mem::replace(&mut self.update_time, now);
            let period = now - last;
            let delay = Instant::now() - now;
            println!("period = {:?}, delay = {:?}", period, delay);
        }
        result
    }
}

impl Server {
    pub const fn new(port: u16) -> Self {
        Self(port)
    }
}

impl<H, E> Recipe<H, E> for Server
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
        let mut buf = [0u8; 1500];
        Box::pin(repeat_with(move || {
            let socket = socket.clone();
            let (len, add) = task::block_on(socket.recv_from(&mut buf)).unwrap();
            Message::MessageReceived(Instant::now(), add, buf[..len].to_vec())
        }))
    }
}

#[test]
fn send() {
    use std::net::{Ipv4Addr, SocketAddrV4};
    task::block_on(async {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        let _ = socket
            .send_to(
                &[1, 2, 3, 4, 5, 6, 7, 8, 9],
                SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 12345)),
            )
            .await;
    });
}
