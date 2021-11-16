use crate::figure::Figure;
use iced::{canvas::*, mouse, Color, Point, Rectangle, Size};
use std::{
    cell::{Cell, RefCell},
    time::Instant,
};

const BORDER_OFFSET: Point = Point { x: 64.0, y: 32.0 };

pub struct FigureCanvas {
    border_cache: Cache,

    time: Cell<Instant>,
    figure: RefCell<Figure>,
}

impl FigureCanvas {
    pub fn new() -> Self {
        Self {
            border_cache: Default::default(),
            time: Cell::new(Instant::now()),
            figure: RefCell::new(Figure::new()),
        }
    }
}

impl<Message> Program<Message> for FigureCanvas {
    fn draw(&self, bounds: iced::Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let now = Instant::now();

        // 绘制数据
        let (rectangle, mut topics) = self
            .figure
            .borrow_mut()
            .draw(bounds.size(), available_size(bounds.size()));
        // 绘制边框
        let border = self.border_cache.draw(bounds.size(), |frame| {
            let Size { width, height } = frame.size();
            frame.stroke(
                &Path::rectangle(
                    BORDER_OFFSET,
                    Size {
                        width: width - BORDER_OFFSET.x * 2.0,
                        height: height - BORDER_OFFSET.y * 2.0,
                    },
                ),
                Stroke {
                    color: Color::BLACK,
                    width: 2.0,
                    ..Default::default()
                },
            );
        });
        // 绘制游标
        let mut vernier = Frame::new(bounds.size());
        if let Cursor::Available(p) = cursor {
            mark_cross(&mut vernier, p, rectangle);
        }
        let vernier = vernier.into_geometry();

        let period = now - self.time.get();
        let delay = Instant::now() - now;
        println!("period = {:?}, delay = {:?}", period, delay);
        self.time.set(now);

        topics.extend_from_slice(&[border, vernier]);
        topics
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if let Cursor::Available(p) = cursor {
            if in_bounds_rectangle(bounds.size(), p) {
                return mouse::Interaction::Crosshair;
            }
        }
        mouse::Interaction::default()
    }
}

fn available_size(size: Size) -> Size {
    Size {
        width: size.width - (BORDER_OFFSET.x + 20.0) * 2.0,
        height: size.height - (BORDER_OFFSET.y + 20.0) * 2.0,
    }
}

fn mark_cross(frame: &mut Frame, p: Point, rectangle: Rectangle) {
    let Size { width, height } = frame.size();
    let x = rectangle.x + p.x / width * rectangle.width;
    let y = rectangle.y - p.y / height * rectangle.height;
    if in_bounds_rectangle(frame.size(), p) {
        text(frame, x, p.x + 4.0, 4.0, 24.0);
        text(frame, y, 4.0, p.y - 24.0, 24.0);
        line(frame, 0.0, p.y, p.x - 20.0, p.y, Color::BLACK);
        line(frame, p.x, 0.0, p.x, p.y - 18.0, Color::BLACK);
        line(
            frame,
            p.x + 20.0,
            p.y,
            width - BORDER_OFFSET.x,
            p.y,
            Color::BLACK,
        );
        line(
            frame,
            p.x,
            p.y + 22.0,
            p.x,
            height - BORDER_OFFSET.y,
            Color::BLACK,
        );
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

fn in_bounds_rectangle(size: Size, p: Point) -> bool {
    Rectangle {
        x: BORDER_OFFSET.x,
        y: BORDER_OFFSET.y,
        width: size.width - 2.0 * BORDER_OFFSET.x,
        height: size.height - 2.0 * BORDER_OFFSET.y,
    }
    .contains(p)
}
