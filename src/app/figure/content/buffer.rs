use crate::Vertex;
use iced::Color;
use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

#[derive(Clone)]
pub(crate) struct TopicBuffer {
    pub capacity: usize,                    // 缓存容量
    pub queue: VecDeque<(Instant, Vertex)>, // 点数据
    pub color_map: HashMap<u8, Color>,      // 色彩映射
}

impl Default for TopicBuffer {
    fn default() -> Self {
        Self {
            capacity: 2000,
            queue: Default::default(),
            color_map: Default::default(),
        }
    }
}
