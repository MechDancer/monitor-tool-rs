use super::Pose;
use iced::{Color, Point, Rectangle};
use std::collections::VecDeque;

pub struct Cache {
    pose: Pose,
    max_size: usize,
    connecting: bool,
    display: bool,
    range: Option<Rectangle>,
    cache: VecDeque<(Pose, Color)>,
}

impl Cache {
    pub fn new(pose: Pose, max_size: usize, connecting: bool, display: bool) -> Self {
        Self {
            pose,
            max_size,
            connecting,
            display,
            range: None,
            cache: VecDeque::new(),
        }
    }

    pub fn config(&mut self, pose: Pose, size: usize, connecting: bool, clear: bool) {
        self.pose = pose;
        self.connecting = connecting;
        self.max_size = size;
        self.cache.truncate(size);
        if clear {
            self.cache.clear()
        } else if self.cache.len() > size {
            self.cache.truncate(size);
            self.cache.shrink_to_fit();
        }
    }

    pub fn push(&mut self, color: Color, pose: Pose) {
        if self.cache.len() == self.max_size {
            self.cache.pop_back();
            self.range = None;
        }
        if let Some(rectangle) = self.range {
            if !rectangle.contains(Point {
                x: pose.x,
                y: pose.y,
            }) {
                self.range = None;
            }
        }
        self.cache.push_front((pose, color));
    }
}
