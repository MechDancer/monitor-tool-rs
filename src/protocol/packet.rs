use super::{Camera, Visible, RGBA};
use crate::Vertex;
use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

#[derive(Default)]
pub struct Encoder {
    camera: Camera,
    sync_sets: HashMap<String, SyncSetBody>,
    layers: HashMap<String, LayerBody>,
    topics: HashMap<String, TopicBody>,
}

#[derive(Default)]
struct SyncSetBody {
    index: u16,
    life_time: Duration,
}

#[derive(Default)]
struct LayerBody {
    index: u16,
    visible: Visible,
}

#[derive(Default)]
struct TopicBody {
    sync_set: u16,
    layer: u16,
    capacity: u32,
    clear: bool,
    colors: HashMap<u8, RGBA>,
    vertex: Vec<Vertex>,
}

macro_rules! update_topic {
    ($encoder:expr, $topic:expr; $mut:expr, $new:expr) => {
        match $encoder.topics.entry($topic) {
            Entry::Occupied(mut entry) => {
                $mut(entry.get_mut());
            }
            Entry::Vacant(entry) => {
                entry.insert($new);
            }
        }
    };
}

macro_rules! hash_map {
    ($key:expr, $value:expr) => {{
        let mut map = HashMap::new();
        map.insert($key, $value);
        map
    }};
}

impl Encoder {
    /// 控制摄像机
    #[inline]
    pub fn camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    /// 更新同步组
    pub fn sync_set(
        &mut self,
        set: impl ToString,
        topics: &[impl ToString],
        life_time: Option<Duration>,
    ) {
        if topics.is_empty() && life_time.is_none() {
            return;
        }
        // 获取序号
        let index = self.sync_sets.len() as u16;
        let index = match self.sync_sets.entry(set.to_string()) {
            Entry::Occupied(mut entry) => {
                let body = entry.get_mut();
                if let Some(life_time) = life_time {
                    body.life_time = life_time;
                }
                body.index
            }
            Entry::Vacant(entry) => {
                entry.insert(SyncSetBody {
                    index,
                    life_time: life_time.unwrap_or(Duration::ZERO),
                });
                index
            }
        };
        // 更新序号
        for topic in topics.iter().map(|it| it.to_string()) {
            update_topic!(self, topic;
                |body: &mut TopicBody| body.sync_set = index,
                TopicBody {
                    sync_set: index,
                    ..Default::default()
                }
            )
        }
    }

    /// 更新图层
    pub fn layer(&mut self, layer: impl ToString, topics: &[impl ToString], visible: Option<bool>) {
        if topics.is_empty() && visible.is_none() {
            return;
        }
        // 获取序号
        let index = self.layers.len() as u16;
        let index = match self.layers.entry(layer.to_string()) {
            Entry::Occupied(mut entry) => {
                let body = entry.get_mut();
                if visible.is_some() {
                    body.visible = visible.into();
                }
                body.index
            }
            Entry::Vacant(entry) => {
                entry.insert(LayerBody {
                    index,
                    visible: visible.into(),
                });
                index
            }
        };
        // 更新序号
        for topic in topics.iter().map(|it| it.to_string()) {
            update_topic!(self, topic;
                |body: &mut TopicBody| body.layer = index,
                TopicBody {
                    layer: index,
                    ..Default::default()
                }
            )
        }
    }

    /// 设置话题颜色
    pub fn topic_set_color(&mut self, topic: impl ToString, level: u8, color: RGBA) {
        update_topic!(self, topic.to_string();
            |body: &mut TopicBody| body.colors.insert(level, color),
            {
                TopicBody {
                    colors: hash_map!(level, color),
                    ..Default::default()
                }
            }
        )
    }

    /// 设置话题容量
    pub fn topic_set_capacity(&mut self, topic: impl ToString, capacity: u32) {
        update_topic!(self, topic.to_string();
            |body: &mut TopicBody| body.capacity = capacity,
            TopicBody {
                capacity,
                ..Default::default()
            }
        )
    }

    /// 清空话题缓存
    pub fn topic_clear(&mut self, topic: impl ToString) {
        update_topic!(self, topic.to_string();
            |body: &mut TopicBody| {
                body.clear = true;
                body.vertex.clear();
            },
            TopicBody {
                clear: true,
                ..Default::default()
            }
        )
    }

    /// 保存顶点
    pub fn topic_push(&mut self, topic: impl ToString, vertex: Vertex) {
        update_topic!(self, topic.to_string();
            |body: &mut TopicBody| {
                body.vertex.push(vertex);
            },
            TopicBody {
                vertex: vec![vertex],
                ..Default::default()
            }
        )
    }

    /// 编码
    pub fn encode(self) -> Vec<u8> {
        todo!()
    }
}
