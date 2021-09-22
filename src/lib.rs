extern crate nalgebra as na;

pub mod cache2d;
pub mod canvas2d;
pub mod protocol;

use std::collections::HashMap;

use iced::Color;
use protocol::{output::FrameOutputStream, Config};

struct Painter {
    stream: FrameOutputStream,
    config: HashMap<String, (Config, Color)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub theta: f32,
}

#[derive(Copy, Clone)]
pub enum PolarAxis {
    Top,
    Left,
}

#[derive(Copy, Clone)]
pub enum BorderMode {
    Rectangular,
    Polar(PolarAxis),
}

impl Painter {
    pub fn new(title: &str, mode: BorderMode) -> Self {
        Self {
            stream: FrameOutputStream::new(title, mode),
            config: HashMap::new(),
        }
    }

    pub fn consume(&mut self) -> Vec<u8> {
        let mut other = FrameOutputStream::new(self.stream.title(), self.stream.mode());
        std::mem::swap(&mut self.stream, &mut other);
        other.to_vec()
    }

    pub fn save_config(&mut self, topic: &str, config: Config, color: Color) {
        self.config.insert(String::from(topic), (config, color));
    }

    pub fn paint_pose(&mut self, topic: &str, poses: &[Pose]) {
        let (config, color) = match self.config.get(topic) {
            Some(c) => c.clone(),
            None => {
                let config = Config::default();
                let color = Color {
                    r: rand::random::<f32>(),
                    g: rand::random::<f32>(),
                    b: rand::random::<f32>(),
                    a: rand::random::<f32>() * 0.75 + 0.25,
                };
                self.save_config(topic, config.clone(), color.clone());
                (config, color)
            }
        };
        let mut stream = self.stream.push_topic(topic, &config, color);
        for p in poses {
            stream.push_pose(*p);
        }
    }

    pub fn paint_xy(&mut self, topic: &str, x: f32, y: f32) {
        self.paint_pose(
            topic,
            &[Pose {
                x,
                y,
                theta: f32::NAN,
            }],
        )
    }
}
