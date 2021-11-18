use async_std::{channel::Sender, net::UdpSocket, sync::Arc, task};
use iced::{
    canvas::{event, Cursor, Event, Geometry, Program},
    futures::stream::{repeat_with, BoxStream},
    mouse, Point, Rectangle, Size,
};
use iced_futures::subscription::Recipe;
use std::time::Instant;

mod figure;

pub(crate) use figure::Figure;
use figure::{is_available, mark_cross};

#[derive(Clone)]
pub struct FigureProgram(pub Sender<FigureEvent>, pub Rectangle, pub Vec<Geometry>);

pub struct UdpReceiver(u16);

#[derive(Debug)]
pub enum Message {
    MessageReceived(Instant, Vec<u8>),
    ViewUpdated,
}

pub enum FigureEvent {
    Zoom(Point, Rectangle, f32),
    Resize(Size),
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

        use mouse::{Event::*, ScrollDelta};
        match event {
            event::Event::Mouse(mouse_event) => match mouse_event {
                WheelScrolled {
                    delta: ScrollDelta::Lines { x: _, y } | ScrollDelta::Pixels { x: _, y },
                } => {
                    let _ = task::block_on(self.0.send(FigureEvent::Zoom(pos, bounds, y)));
                }
                _ => {}
            },
            _ => {}
        }
        (event::Status::Ignored, None)
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let _ = task::block_on(self.0.send(FigureEvent::Resize(bounds.size())));
        let mut geometries = self.2.clone();
        if let Cursor::Available(p) = cursor {
            if is_available(bounds, p) {
                geometries.push(mark_cross(bounds, p, self.1));
            }
        }
        geometries
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if let Cursor::Available(p) = cursor {
            if is_available(bounds, p) {
                return mouse::Interaction::Crosshair;
            }
        }
        mouse::Interaction::default()
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
