use iced::{
    canvas::{Cache, Cursor, Geometry, Program},
    mouse::Interaction,
    Color, Rectangle,
};
use std::{
    cell::{Cell, RefCell},
    time::Instant,
};

mod figure;
mod figure_canvas;

use figure::Figure;
use figure_canvas::*;

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
        let size = bounds.size();
        // 绘制数据
        let (rectangle, mut geometries) = self.figure.borrow_mut().draw(size, available_size(size));
        // 绘制边框
        geometries.push(
            self.border_cache
                .draw(size, |frame| border(frame, Color::BLACK)),
        );
        // 绘制游标
        if let Cursor::Available(p) = cursor {
            if is_available(size, p) {
                geometries.push(mark_cross(size, p, rectangle));
            }
        }
        // 计时
        {
            let period = now - self.time.get();
            let delay = Instant::now() - now;
            println!("period = {:?}, delay = {:?}", period, delay);
            self.time.set(now);
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
