use async_std::{
    channel::{unbounded, Receiver, RecvError, TryRecvError},
    task::{self, JoinHandle},
};
use iced::{canvas::Geometry, Point, Rectangle};

use crate::{Figure, FigureEvent};

pub fn spawn_background(input: Receiver<FigureEvent>) -> Receiver<(Rectangle, Vec<Geometry>)> {
    let (sender, output) = unbounded();
    task::spawn(async move {
        let mut cache = Some(Box::new(Figure::new()));
        loop {
            match input.recv().await {
                Ok(event) => {
                    let figure = cache.take().unwrap();
                    cache = Some(handle(figure, event).await);
                }
                Err(RecvError) => panic!("Impossible!"),
            }
            loop {
                use TryRecvError::*;
                match input.try_recv() {
                    Ok(event) => {
                        let figure = cache.take().unwrap();
                        cache = Some(handle(figure, event).await);
                    }
                    Err(Empty) => break,
                    Err(Closed) => panic!("Impossible!"),
                }
            }
            let _ = sender.send(cache.as_mut().unwrap().draw()).await;
        }
    });
    output
}

fn handle(mut figure: Box<Figure>, event: FigureEvent) -> JoinHandle<Box<Figure>> {
    task::spawn_blocking(move || {
        use FigureEvent::*;
        match event {
            Zoom(pos, bounds, level) => figure.zoom(level, pos, bounds.size()),
            Resize(bounds) => figure.zoom(0.0, Point::ORIGIN, bounds),
        }
        figure
    })
}
