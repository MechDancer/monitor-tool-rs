use iced::Color;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    net::IpAddr,
    time::Instant,
};

/// 话题标题，用于区分话题
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct TopicTitle {
    title: String,
    source: IpAddr,
}

/// 话题内容，用于存储话题状态
#[derive(Default)]
pub struct TopicContent {
    queue: VecDeque<(Instant, Point)>,
    color_map: HashMap<u8, Color>,
}

/// 图形顶点
pub struct Point {
    x: f32,
    y: f32,
    theta: f32,
    level: u8,
    tie: bool,
}

impl Display for TopicTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.title, self.source)
    }
}

#[test]
fn assert_size() {
    assert_eq!(16, std::mem::size_of::<Point>());
}
