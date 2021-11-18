use std::time::Instant;

use async_std::{
    channel::{Receiver, Sender},
    sync::{Arc, Mutex},
    task,
};
use iced::{
    canvas::{event, Cursor, Event, Geometry, Program},
    futures::stream::{repeat_with, BoxStream},
    mouse, Point, Rectangle,
};
use iced_futures::subscription::Recipe;

mod figure;

pub(crate) use figure::Figure;
use figure::{is_available, mark_cross};

#[derive(Clone)]
pub struct FigureProgram {
    pub sender: Sender<FigureEvent>,
    pub state: (Rectangle, Vec<Geometry>),
    bounds: Arc<Mutex<Rectangle>>,
}

pub struct CacheComplete(pub Receiver<(Rectangle, Vec<Geometry>)>);

#[derive(Debug)]
pub enum FigureEvent {
    Zoom(Point, Rectangle, f32),
    Resize(Rectangle),
    Packet(Instant, Vec<u8>),
    Line(String),
}

impl FigureProgram {
    pub fn new(sender: Sender<FigureEvent>) -> Self {
        Self {
            sender,
            state: Default::default(),
            bounds: Default::default(),
        }
    }
}

impl Program<(Rectangle, Vec<Geometry>)> for FigureProgram {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<(Rectangle, Vec<Geometry>)>) {
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
                    let _ = task::block_on(self.sender.send(FigureEvent::Zoom(pos, bounds, y)));
                }
                _ => {}
            },
            _ => {}
        }
        (event::Status::Ignored, None)
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let mut memory = task::block_on(self.bounds.lock());
        if bounds != *memory {
            *memory = bounds;
            let _ = task::block_on(self.sender.send(FigureEvent::Resize(bounds)));
        }
        let mut geometries = self.state.1.clone();
        if let Cursor::Available(p) = cursor {
            if is_available(bounds, p) {
                geometries.push(mark_cross(bounds, p, self.state.0));
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

impl<H, E> Recipe<H, E> for CacheComplete
where
    H: std::hash::Hasher,
{
    type Output = (Rectangle, Vec<Geometry>);

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, E>) -> BoxStream<'static, Self::Output> {
        Box::pin(repeat_with(move || task::block_on(self.0.recv()).unwrap()))
    }
}
