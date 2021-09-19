use super::Pose;
use iced::Color;
use std::collections::VecDeque;

pub struct Points {
    pose: Pose,
    max_size: usize,
    connecting: bool,
    cache: VecDeque<(Pose, Color)>,
}

impl Points {
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
        }
        self.cache.push_front((pose, color));
    }
}
