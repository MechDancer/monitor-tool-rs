use super::{figure::FigureSnapshot, figure_program::FigureEvent, Figure};
use crate::protocol::decode;
use async_std::{
    channel::{unbounded, Receiver, RecvError, TryRecvError},
    task::{self, JoinHandle},
};
use iced::{canvas::Geometry, Point, Rectangle};

pub fn spawn_background(
    input: Receiver<FigureEvent>,
    resume: Option<(Point, FigureSnapshot)>,
) -> Receiver<(Rectangle, Vec<Geometry>)> {
    let (sender, output) = unbounded();
    task::spawn(async move {
        let mut cache = if let Some((center, snapshot)) = resume {
            let mut result = Box::new(Figure::from(snapshot));
            result.set_view(center.x, center.y, f32::NAN, f32::NAN);
            Some(result)
        } else {
            Some(Default::default())
        };
        loop {
            match input.recv().await {
                Ok(event) => {
                    let figure = cache.take().unwrap();
                    cache = Some(handle(figure, event).await);
                }
                Err(RecvError) => return,
            }
            loop {
                use TryRecvError::*;
                match input.try_recv() {
                    Ok(event) => {
                        let figure = cache.take().unwrap();
                        cache = Some(handle(figure, event).await);
                    }
                    Err(Empty) => break,
                    Err(Closed) => return,
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
            Auto => figure.auto_view = true,
            Zoom(pos, bounds, level) => figure.zoom(level, pos, bounds),
            Resize(bounds) => figure.zoom(0.0, Point::ORIGIN, bounds),
            ReadyForGrab => figure.auto_view = false,
            Grab(v) => figure.grab(v),
            Select(bounds, p0, p1) => figure.select(bounds, p0, p1),
            Packet(time, buf) => decode(figure.as_mut(), time, buf.as_slice()),
            Line(line) => {
                let words = line.split_whitespace().collect::<Vec<_>>();
                match words.as_slice() {
                    ["log", "time"] => figure.set_print_time(true),
                    ["unlog", "time"] => figure.set_print_time(false),
                    ["save", path] => {
                        let snapshot = figure.snapshot();
                        task::spawn(snapshot.save(path.into()));
                    }
                    ["goto", coordinate] => {
                        let mut coordinate = coordinate.split(',');
                        let x: Option<f32> = coordinate.next().and_then(|s| s.parse().ok());
                        let y: Option<f32> = coordinate.next().and_then(|s| s.parse().ok());
                        if let (Some(x), Some(y)) = (x, y) {
                            figure.set_view(x, y, f32::NAN, f32::NAN);
                        }
                    }
                    ["show", layer] => figure.set_visible(layer, true),
                    ["hide", layer] => figure.set_visible(layer, false),
                    [topic, "focus", num] => {
                        if let Ok(n) = num.parse() {
                            if let Some(content) = figure.get_topic(topic) {
                                content.set_focus(n);
                                println!("set focus {} for {}", n, topic);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        figure
    })
}
