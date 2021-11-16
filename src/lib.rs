use async_std::{
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    task,
};
use iced::{
    canvas::{Cache, Cursor, Geometry, Program},
    futures::stream::{repeat_with, BoxStream},
    mouse::Interaction,
    Color, Rectangle, Size,
};
use iced_futures::subscription::Recipe;
use std::time::Instant;

mod figure;
mod figure_canvas;

use figure::Figure;
use figure_canvas::*;

#[derive(Clone)]
pub struct FigureProgram(Arc<Mutex<FigureCanvas>>);

pub struct Server(u16);

struct FigureCanvas {
    update_time: Instant,
    border_cache: Cache,
    figure: Figure,
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
    pub async fn receive(&self, buf: &[u8]) {
        self.0.lock().await.figure.receive(buf);
    }
}

impl<Message> Program<Message> for FigureProgram {
    fn draw(&self, bounds: iced::Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let size = bounds.size();
        let (rectangle, mut geometries) = task::block_on(async { self.0.lock().await.draw(size) });
        if let Cursor::Available(p) = cursor {
            if is_available(size, p) {
                geometries.push(mark_cross(size, p, rectangle));
            }
        }
        geometries
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> Interaction {
        if let Cursor::Available(p) = cursor {
            if is_available(bounds.size(), p) {
                return Interaction::Crosshair;
            }
        }
        Interaction::default()
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
    pub fn new(port: u16) -> Self {
        // Self(Arc::new(
        //     task::block_on(UdpSocket::bind(format!("0.0.0.0:{}", port))).unwrap(),
        // ))
        Self(port)
    }
}

impl<H, E> Recipe<H, E> for Server
where
    H: std::hash::Hasher,
{
    type Output = (std::time::Instant, SocketAddr, Vec<u8>);

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
            task::block_on(async move {
                let (len, add) = socket.recv_from(&mut buf).await.unwrap();
                (Instant::now(), add, buf[..len].to_vec())
            })
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
