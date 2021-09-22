use std::collections::HashMap;

use iced::{canvas::*, mouse, Color, Point, Rectangle, Size};

const BORDER_OFFSET: Point = Point { x: 64.0, y: 32.0 };

pub struct DrawState<'a> {
    border_cache: Cache,
    title: &'a str,
    topics: HashMap<String, ()>,
}

impl<'a> DrawState<'a> {
    pub fn new(title: &'a str) -> Self {
        DrawState {
            border_cache: Default::default(),
            title,
            topics: Default::default(),
        }
    }

    pub fn title(&self) -> &'a str {
        self.title
    }
}

impl<Message> Program<Message> for DrawState<'_> {
    fn draw(&self, bounds: iced::Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());

        let border = self.border_cache.draw(bounds.size(), |frame| {
            let path = Path::rectangle(
                BORDER_OFFSET,
                Size {
                    width: frame.size().width - BORDER_OFFSET.x * 2.0,
                    height: frame.size().height - BORDER_OFFSET.y * 2.0,
                },
            );
            frame.stroke(
                &path,
                Stroke {
                    color: Color::BLACK,
                    ..Default::default()
                },
            );
        });

        if let Cursor::Available(p) = cursor {
            mark_cross(&mut frame, bounds.size(), p);
        }
        vec![border, frame.into_geometry()]
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if let Cursor::Available(p) = cursor {
            if in_bound(bounds.size(), p) {
                return mouse::Interaction::Crosshair;
            }
        }
        mouse::Interaction::default()
    }
}

fn line(frame: &mut Frame, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let line = Path::line(Point { x: x0, y: y0 }, Point { x: x1, y: y1 });
    frame.stroke(
        &line,
        Stroke {
            color,
            width: 1.0,
            ..Default::default()
        },
    );
}

fn text(frame: &mut Frame, num: f32, x: f32, y: f32, size: f32) {
    frame.fill_text(Text {
        content: format!("{:.2}", num),
        position: Point { x, y },
        color: Color::BLACK,
        size,
        ..Default::default()
    });
}

fn in_bound(bounds: Size, p: Point) -> bool {
    BORDER_OFFSET.x < p.x
        && p.x < bounds.width - BORDER_OFFSET.x
        && BORDER_OFFSET.y < p.y
        && p.y < bounds.height - BORDER_OFFSET.y
}

fn mark_cross(frame: &mut Frame, bounds: Size, p: Point) {
    if in_bound(bounds, p) {
        text(frame, p.x, p.x + 4.0, 4.0, 24.0);
        text(frame, p.y, 4.0, p.y - 24.0, 24.0);
        line(frame, 0.0, p.y, p.x - 20.0, p.y, Color::BLACK);
        line(frame, p.x, 0.0, p.x, p.y - 18.0, Color::BLACK);
        line(
            frame,
            p.x + 20.0,
            p.y,
            bounds.width - BORDER_OFFSET.x,
            p.y,
            Color::BLACK,
        );
        line(
            frame,
            p.x,
            p.y + 22.0,
            p.x,
            bounds.height - BORDER_OFFSET.y,
            Color::BLACK,
        );
    }
}
