use crate::{decode, Figure, FigureEvent};
use async_std::{
    channel::{unbounded, Receiver, RecvError, TryRecvError},
    task::{self, JoinHandle},
};
use iced::{canvas::Geometry, Point, Rectangle};

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
            Zoom(pos, bounds, level) => figure.zoom(level, pos, bounds),
            Resize(bounds) => figure.zoom(0.0, Point::ORIGIN, bounds),
            ReadyForGrab => figure.auto_view = false,
            Grab(v) => figure.grab(v),
            Packet(time, buf) => decode(figure.as_mut(), time, buf.as_slice()),
            Line(line) => {
                let words = line.split_whitespace().collect::<Vec<_>>();
                match words.as_slice() {
                    ["auto"] => figure.set_view(f32::NAN, f32::NAN, 0.0, 0.0),
                    ["log", "time"] => figure.set_print_time(true),
                    ["unlog", "time"] => figure.set_print_time(false),
                    [title, "focus", num] => {
                        if let Ok(n) = num.parse() {
                            if let Some(topic) = figure.get_topic(&title.to_string()) {
                                topic.set_focus(n);
                                println!("set focus {} for {}", n, title);
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
