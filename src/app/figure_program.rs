use super::figure::{as_available, mark_anchor, mark_cross};
use async_std::{
    channel::{Receiver, Sender},
    sync::{Arc, Mutex},
    task,
};
use iced::{
    canvas::{event, Cursor, Event, Frame, Geometry, Program},
    futures::stream::{repeat_with, BoxStream},
    keyboard, mouse, Color, Point, Rectangle, Vector,
};
use iced_futures::subscription::Recipe;
use std::{
    sync::atomic::{AtomicBool, Ordering::Relaxed},
    time::Instant,
};

#[derive(Clone)]
pub struct FigureProgram {
    pub sender: Sender<FigureEvent>,
    pub state: (Rectangle, Vec<Geometry>),
    bounds: Arc<Mutex<Rectangle>>,
    anchor: Arc<Mutex<Anchor>>,
    dark_mode: Arc<AtomicBool>,
}

/// 订阅新近完成的图像缓存
pub struct CacheComplete(pub Receiver<(Rectangle, Vec<Geometry>)>);

#[derive(Debug)]
pub enum FigureEvent {
    Auto,
    Zoom(Point, Rectangle, f32),
    Resize(Rectangle),
    ReadyForGrab,
    Grab(Vector),
    Select(Rectangle, Point, Point),
    Packet(Instant, Vec<u8>),
    Line(String),
}

#[derive(Default, Clone, Copy, Debug)]
struct Anchor {
    pub pos: Point,
    pub which: Option<mouse::Button>,
}

impl FigureProgram {
    pub fn new(sender: Sender<FigureEvent>) -> Self {
        Self {
            sender,
            state: Default::default(),
            bounds: Default::default(),
            anchor: Default::default(),
            dark_mode: Arc::new(AtomicBool::new(true)),
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
        let pos = if let Some(pos) = as_available(bounds, cursor) {
            pos
        } else {
            task::block_on(self.anchor.lock()).which = None;
            return (event::Status::Ignored, None);
        };

        use keyboard::{Event::*, KeyCode::Space};
        use mouse::{Button::*, Event::*, ScrollDelta};
        match event {
            event::Event::Keyboard(KeyPressed {
                key_code: Space,
                modifiers: _,
            }) => {
                self.send(FigureEvent::Auto);
            }
            event::Event::Keyboard(_) => {}
            event::Event::Mouse(mouse_event) => match mouse_event {
                WheelScrolled {
                    delta: ScrollDelta::Lines { x: _, y } | ScrollDelta::Pixels { x: _, y },
                } => {
                    self.send(FigureEvent::Zoom(pos, bounds, y));
                }
                ButtonPressed(b) => match b {
                    Left | Right => {
                        *task::block_on(self.anchor.lock()) = Anchor {
                            pos,
                            which: Some(b),
                        };
                        self.send(FigureEvent::ReadyForGrab);
                    }
                    _ => {}
                },
                ButtonReleased(b) => match b {
                    Left => {
                        let mut anchor = task::block_on(self.anchor.lock());
                        if anchor.which == Some(Left) {
                            anchor.which = None;
                        }
                    }
                    Right => {
                        let mut anchor = task::block_on(self.anchor.lock());
                        if anchor.which == Some(Right) {
                            anchor.which = None;
                            self.send(FigureEvent::Select(bounds, anchor.pos, pos));
                        }
                    }
                    _ => {}
                },
                CursorMoved { position: _ } => {
                    let mut anchor = task::block_on(self.anchor.lock());
                    if let Some(Left) = anchor.which {
                        self.send(FigureEvent::Grab(
                            pos - std::mem::replace(&mut anchor.pos, pos),
                        ));
                    }
                }
                _ => {}
            },
        }
        (event::Status::Ignored, None)
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let pos = as_available(bounds, cursor);
        // 画图像
        let mut geometries = self.state.1.clone();
        // 响应 resize
        let mut anchor = task::block_on(self.anchor.lock());
        if bounds != std::mem::replace(&mut *task::block_on(self.bounds.lock()), bounds) {
            self.send(FigureEvent::Resize(bounds));
            if anchor.which.is_some() {
                if let Some(pos) = pos {
                    anchor.pos = pos;
                } else {
                    anchor.which = None;
                }
            }
        }
        if let Some(p) = as_available(bounds, cursor) {
            let color = if self.dark_mode.load(Relaxed) {
                Color::WHITE
            } else {
                Color::BLACK
            };
            // 画光标
            let mut frame = Frame::new(bounds.size());
            mark_cross(&mut frame, bounds, p, self.state.0, color);
            // 画候选框
            if anchor.which == Some(mouse::Button::Right) {
                mark_anchor(&mut frame, anchor.pos, p, color);
            }
            geometries.push(frame.into_geometry());
        }
        geometries
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        use mouse::Interaction;
        if as_available(bounds, cursor).is_some() {
            if task::block_on(self.anchor.lock()).which == Some(mouse::Button::Left) {
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
