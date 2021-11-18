use super::{Camera, Visible, RGBA};
use crate::Vertex;
use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

#[derive(Default)]
pub struct Encoder {
    camera: Camera,
    sync_sets: HashMap<String, WithIndex<Duration>>,
    layers: HashMap<String, WithIndex<Visible>>,
    topics: HashMap<String, TopicBody>,
}

#[derive(Default)]
struct WithIndex<T: Default> {
    index: u16,
    value: T,
}

#[derive(Default)]
struct TopicBody {
    sync_set: u16,
    layer: u16,
    clear: bool,
    capacity: u32,
    focus: u32,
    colors: HashMap<u8, RGBA>,
    vertex: Vec<Vertex>,
}

#[inline]
fn bytes_of<'a, T>(t: &'a T) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(t as *const _ as *const u8, std::mem::size_of::<T>()) }
}

macro_rules! extend {
    (     $value:expr => $vec:expr) => { $vec.extend_from_slice(bytes_of(&$value)) };
    (str; $value:expr => $vec:expr) => { $vec.extend_from_slice($value.as_bytes()) };
    (len; $value:expr => $vec:expr) => { extend!($value as u16 => $vec) };
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
        let next = self.sync_sets.len() as u16;
        let body = self
            .sync_sets
            .entry(set.to_string())
            .or_insert_with(|| WithIndex {
                index: next + 1,
                value: Default::default(),
            });
        if let Some(life_time) = life_time {
            body.value = life_time;
        }
        // 更新序号
        for topic in topics.iter().map(|it| it.to_string()) {
            self.topics.entry(topic).or_default().sync_set = body.index;
        }
    }

    /// 更新图层
    pub fn layer(&mut self, layer: impl ToString, topics: &[impl ToString], visible: Option<bool>) {
        if topics.is_empty() && visible.is_none() {
            return;
        }
        // 获取序号
        let next = self.layers.len() as u16;
        let body = self
            .layers
            .entry(layer.to_string())
            .or_insert_with(|| WithIndex {
                index: next + 1,
                value: Default::default(),
            });
        body.value = visible.into();
        // 更新序号
        for topic in topics.iter().map(|it| it.to_string()) {
            self.topics.entry(topic).or_default().layer = body.index;
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

    /// 设置话题容量
    pub fn topic_set_focus(&mut self, topic: impl ToString, focus: u32) {
        self.topics.entry(topic.to_string()).or_default().focus = focus;
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
        let mut buf = Vec::new();
        // 编码摄像机配置
        extend!(self.camera => buf);
        // 编码同步组
        sort_and_encode(&self.sync_sets, &mut buf);
        // 编码图层
        sort_and_encode(&self.layers, &mut buf);
        // 编码话题
        for (name, body) in self.topics {
            extend!(len; name.len()    => buf);
            extend!(str; name          => buf);
            extend!(     body.sync_set => buf);
            extend!(     body.layer    => buf);
            extend!(     body.clear    => buf);
            extend!(     body.capacity => buf);
            extend!(     body.focus    => buf);
            // 编码颜色
            extend!(len; body.colors.len() => buf);
            for (level, rgba) in body.colors {
                extend!(level => buf);
                extend!(rgba  => buf);
            }
            // 编码顶点
            extend!(len; body.vertex.len() => buf);
            for v in body.vertex {
                extend!(v => buf);
            }
        }
        println!("{}", buf.len());
        buf
    }
}

/// 从已排序的集合编码
fn sort_and_encode<T: Default>(map: &HashMap<String, WithIndex<T>>, buf: &mut Vec<u8>) {
    // 用 u16 保存长度
    const USIZE_LEN: usize = std::mem::size_of::<u16>();
    // 依序号排序
    let mut sorted = vec![None; map.len()];
    for (name, body) in map {
        sorted[body.index as usize] = Some((name.as_str(), &body.value));
    }
    // 编码：| 数量 n | 每个尾部位置 × n | 逐个编码 |
    extend!(len; sorted.len() => buf);
    let mut ptr_len = buf.len();
    let ptr_content = ptr_len + USIZE_LEN * map.len();
    buf.resize(ptr_content, 0);
    sorted
        .into_iter()
        .map(|o| o.unwrap())
        .for_each(|(name, body)| {
            extend!(     *body => buf);
            extend!(str;  name => buf);
            unsafe {
                *(buf[ptr_len..].as_ptr() as *mut _) = (buf.len() - ptr_content) as u16;
            }
            ptr_len += USIZE_LEN;
        });
}

#[test]
fn send() {
    use async_std::{net::UdpSocket, task};

    let mut encoder = Encoder::default();
    encoder.topic_clear("test");
    for i in 0..100 {
        let x = (i as f32) * 0.1 + 20.0;
        encoder.topic_push(
            "test",
            Vertex {
                x,
                y: x.sin(),
                dir: f32::NAN,
                level: 0,
                tie: true,
                _zero: 0,
            },
        )
    }
    let buf = encoder.encode();
    let socket = task::block_on(UdpSocket::bind("0.0.0.0:0")).unwrap();
    let _ = task::block_on(socket.send_to(&buf, "127.0.0.1:12345"));
}
