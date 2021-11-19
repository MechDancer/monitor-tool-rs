use std::time::Instant;

use async_std::{
    channel::{Receiver, Sender},
    sync::{Arc, Mutex},
    task,
};
use iced::{
    canvas::{event, Cursor, Event, Geometry, Program},
    futures::stream::{repeat_with, BoxStream},
    mouse::{self, Button},
    Point, Rectangle, Vector,
};
use iced_futures::subscription::Recipe;

mod anchor;
mod figure;

use anchor::Anchor;
pub(crate) use figure::Figure;
use figure::{as_available, mark_cross};

#[derive(Clone)]
pub struct FigureProgram {
    pub sender: Sender<FigureEvent>,
    pub state: (Rectangle, Vec<Geometry>),
    bounds: Arc<Mutex<Rectangle>>,
    anchor: Arc<Mutex<Anchor>>,
}

pub struct CacheComplete(pub Receiver<(Rectangle, Vec<Geometry>)>);

#[derive(Debug)]
pub enum FigureEvent {
    Zoom(Point, Rectangle, f32),
    Resize(Rectangle),
    ReadyForGrab,
    Grab(Vector),
    Packet(Instant, Vec<u8>),
    Line(String),
}

impl FigureProgram {
    pub fn new(sender: Sender<FigureEvent>) -> Self {
        Self {
            sender,
            state: Default::default(),
            bounds: Default::default(),
            anchor: Default::default(),
        }
    }

    #[inline]
    fn send(&self, event: FigureEvent) {
        let _ = task::block_on(self.sender.send(event));
    }
}

impl Program<(Rectangle, Vec<Geometry>)> for FigureProgram {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<(Rectangle, Vec<Geometry>)>) {
        let pos = if let Some(p) = as_available(bounds, cursor) {
            p
        } else {
            return (event::Status::Ignored, None);
        };

        use mouse::{Button::*, Event::*, ScrollDelta};
        match event {
            event::Event::Mouse(mouse_event) => match mouse_event {
                WheelScrolled {
                    delta: ScrollDelta::Lines { x: _, y } | ScrollDelta::Pixels { x: _, y },
                } => {
                    self.send(FigureEvent::Zoom(pos, bounds, y));
                }
                ButtonPressed(b) => match b {
                    Left => {
                        *task::block_on(self.anchor.lock()) = Anchor {
                            bounds,
                            pos,
                            which: Some(b),
                        };
                        self.send(FigureEvent::ReadyForGrab);
                    }
                    _ => {}
                },
                ButtonReleased(b) => match b {
                    Left => {
                        task::block_on(self.anchor.lock()).which = None;
                    }
                    _ => {}
                },
                CursorMoved { position: _ } => {
                    let mut anchor = task::block_on(self.anchor.lock());
                    if anchor.which == Some(Button::Left) {
                        self.send(FigureEvent::Grab(pos - anchor.pos));
                        anchor.pos = pos;
                    }
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
            self.send(FigureEvent::Resize(bounds));
        }
        let mut geometries = self.state.1.clone();
        if let Some(p) = as_available(bounds, cursor) {
            geometries.push(mark_cross(bounds, p, self.state.0));
        }
        geometries
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        use mouse::Interaction;
        if as_available(bounds, cursor).is_some() {
            if task::block_on(self.anchor.lock()).which == Some(Button::Left) {
                Interaction::Grab
            } else {
                Interaction::Crosshair
            }
        } else {
            Interaction::default()
        }
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
