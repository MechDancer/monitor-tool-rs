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
    clear: bool,
    capacity: u32,
    colors: HashMap<u8, RGBA>,
    vertex: Vec<Vertex>,
}

const USIZE_LEN: usize = std::mem::size_of::<u16>();

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
            self.topics.entry(topic).or_default().sync_set = index;
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
            self.topics.entry(topic).or_default().layer = index;
        }
    }

    /// 设置话题颜色
    pub fn topic_set_color(&mut self, topic: impl ToString, level: u8, color: RGBA) {
        self.topics
            .entry(topic.to_string())
            .or_default()
            .colors
            .insert(level, color);
    }

    /// 设置话题容量
    pub fn topic_set_capacity(&mut self, topic: impl ToString, capacity: u32) {
        self.topics.entry(topic.to_string()).or_default().capacity = capacity;
    }

    /// 清空话题缓存
    pub fn topic_clear(&mut self, topic: impl ToString) {
        match self.topics.entry(topic.to_string()) {
            Entry::Occupied(mut entry) => {
                let topic = entry.get_mut();
                topic.clear = true;
                topic.vertex.clear();
            }
            Entry::Vacant(entry) => {
                entry.insert(TopicBody {
                    clear: true,
                    ..Default::default()
                });
            }
        }
    }

    /// 保存顶点
    pub fn topic_push(&mut self, topic: impl ToString, vertex: Vertex) {
        self.topics
            .entry(topic.to_string())
            .or_default()
            .vertex
            .push(vertex);
    }

    /// 编码
    pub fn encode(self) -> Vec<u8> {
        let mut result = Vec::new();
        // 编码摄像机配置
        extend_bytes(&self.camera, &mut result);
        // 编码同步组
        let mut sorted = vec![None; self.sync_sets.len()];
        for (name, body) in &self.sync_sets {
            sorted[body.index as usize] = Some((name.as_str(), &body.life_time));
        }
        extend_from_sorted(sorted, &mut result);
        // 编码图层
        let mut sorted = vec![None; self.layers.len()];
        for (name, body) in &self.layers {
            sorted[body.index as usize] = Some((name.as_str(), &body.visible));
        }
        extend_from_sorted(sorted, &mut result);
        // 编码话题
        for (name, body) in self.topics {
            extend_bytes(&(name.len() as u16), &mut result);
            result.extend_from_slice(name.as_bytes());
            extend_bytes(&body.sync_set, &mut result);
            extend_bytes(&body.layer, &mut result);
            extend_bytes(&body.clear, &mut result);
            extend_bytes(&body.capacity, &mut result);
            extend_bytes(&(body.colors.len() as u16), &mut result);
            for (level, rgba) in body.colors {
                extend_bytes(&level, &mut result);
                extend_bytes(&rgba, &mut result);
            }
            extend_bytes(&(body.vertex.len() as u16), &mut result);
            for v in body.vertex {
                extend_bytes(&v, &mut result);
            }
        }
        result
    }
}

#[inline]
fn bytes_of<'a, T>(t: &'a T) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(t as *const _ as *const u8, std::mem::size_of::<T>()) }
}

#[inline]
fn extend_bytes<T>(value: &T, to: &mut Vec<u8>) {
    to.extend_from_slice(bytes_of(value));
}

/// 从已排序的集合编码
fn extend_from_sorted<T>(sorted: Vec<Option<(&str, &T)>>, result: &mut Vec<u8>) {
    result.extend_from_slice(bytes_of(&(sorted.len() as u16)));
    let mut ptr_len = result.len();
    let ptr_content = ptr_len + USIZE_LEN * result.len();
    result.resize(ptr_content, 0);
    sorted
        .into_iter()
        .map(|o| o.unwrap())
        .for_each(|(name, body)| {
            result.extend_from_slice(bytes_of(body));
            result.extend_from_slice(name.as_bytes());
            unsafe {
                *(result[ptr_len..].as_ptr() as *mut _) = (result.len() - ptr_content) as u16;
            }
            ptr_len += USIZE_LEN;
        });
}
