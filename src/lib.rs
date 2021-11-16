use async_std::{
    sync::{Arc, Mutex},
    task,
};
use iced::{
    canvas::{Cache, Cursor, Geometry, Program},
    mouse::Interaction,
    Color, Rectangle, Size,
};
use std::time::Instant;

mod figure;
mod figure_canvas;

use figure::Figure;
use figure_canvas::*;

pub struct FigureCanvas {
    update_time: Instant,
    border_cache: Cache,
    figure: Figure,
}

impl FigureCanvas {
    pub fn new() -> Self {
        Self {
            update_time: Instant::now(),
            border_cache: Default::default(),
            figure: Figure::new(),
        }
    }

    pub fn draw(&mut self, size: Size) -> (Rectangle, Vec<Geometry>) {
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

#[derive(Clone)]
pub struct FigureProgram(Arc<Mutex<FigureCanvas>>);

impl FigureProgram {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(FigureCanvas::new())))
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
