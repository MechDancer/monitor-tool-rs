use super::Visible;
use crate::Vertex;
use palette::{rgb::channels::Argb, Packed, Srgba};
use std::{collections::HashMap, time::Duration};

#[derive(Default)]
pub struct Encoder {
    sync_sets: HashMap<String, WithIndex<Duration>>,
    layers: HashMap<String, WithIndex<Visible>>,
    topics: HashMap<String, TopicBody>,
}

pub struct TopicEncoder<'a>(&'a mut TopicBody);

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
    colors: HashMap<u8, u32>,
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
    /// 立即编码
    #[inline]
    pub fn with(f: impl FnOnce(&mut Encoder) -> ()) -> Vec<u8> {
        let mut this = Self::default();
        f(&mut this);
        this.encode()
    }

    /// 立即编码
    #[inline]
    pub fn with_topic(&mut self, topic: impl ToString, f: impl FnOnce(TopicEncoder) -> ()) {
        f(self.topic(topic.to_string()));
    }

    /// 配置话题
    pub fn config_topic(
        &mut self,
        topic: impl ToString,
        capacity: u32,
        focus: u32,
        colors: &[(u8, Srgba)],
        f: impl FnOnce(TopicEncoder) -> (),
    ) {
        let mut encoder = self.topic(topic.to_string());
        encoder.set_capacity(capacity);
        encoder.set_focus(focus);
        for (level, color) in colors {
            encoder.set_color(*level, *color);
        }
        f(encoder);
    }

    /// 构造话题编码器
    #[inline]
    pub fn topic<'a>(&'a mut self, topic: impl ToString) -> TopicEncoder<'a> {
        TopicEncoder(self.topics.entry(topic.to_string()).or_default())
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

    /// 编码
    pub fn encode(self) -> Vec<u8> {
        let mut buf = Vec::new();
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
        buf
    }
}

impl<'a> TopicEncoder<'a> {
    /// 设置话题颜色
    #[inline]
    pub fn set_color(&mut self, level: u8, color: Srgba) {
        self.0
            .colors
            .insert(level, Packed::<Argb>::from(color.into_format()).color);
    }

    /// 设置话题容量
    #[inline]
    pub fn set_capacity(&mut self, capacity: u32) {
        self.0.capacity = capacity;
    }

    /// 设置关注数量
    #[inline]
    pub fn set_focus(&mut self, focus: u32) {
        self.0.focus = focus;
    }

    /// 清空话题缓存
    #[inline]
    pub fn clear(&mut self) {
        self.0.vertex.clear();
        self.0.clear = true;
    }

    /// 保存顶点
    #[inline]
    pub fn push(&mut self, vertex: Vertex) {
        self.0.vertex.push(vertex);
    }

    /// 保存一组顶点
    #[inline]
    pub fn extend(&mut self, vertex: impl Iterator<Item = Vertex>) {
        vertex.for_each(|v| self.0.vertex.push(v));
    }

    /// 保存多边形
    #[inline]
    pub fn push_polygon(&mut self, mut vertex: impl Iterator<Item = Vertex>) {
        if let Some(begin) = vertex.next() {
            self.0.vertex.push(Vertex { alpha: 0, ..begin });
            vertex.for_each(|v| self.0.vertex.push(v));
            self.0.vertex.push(begin);
        }
    }
}

/// 从已排序的集合编码
fn sort_and_encode<T: Default>(map: &HashMap<String, WithIndex<T>>, buf: &mut Vec<u8>) {
    // 用 u16 保存长度
    const USIZE_LEN: usize = std::mem::size_of::<u16>();
    // 依序号排序
    let mut sorted = vec![None; map.len()];
    for (name, body) in map {
        sorted[body.index as usize - 1] = Some((name.as_str(), &body.value));
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
    use crate::Shape;
    use palette::Srgba;
    use rand::{thread_rng, Rng};
    use std::net::UdpSocket;

    let mut rng = thread_rng();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    {
        let mut encoder = Encoder::default();
        let mut test = encoder.topic("test");
        test.set_capacity(200000);
        test.set_focus(200);
        test.clear();
        for i in 0..255 {
            test.set_color(i, rng.gen::<Srgba>());
        }
        let _ = socket.send_to(&encoder.encode(), "127.0.0.1:12345");
    }

    for i in 0 as u64.. {
        use std::{f32::consts::PI, thread};
        let mut encoder = Encoder::default();
        let mut test = encoder.topic("test");
        for j in 0..500 {
            let theta = ((i * 500 + j) as f32).powf(1.1) * PI * 1e-2;
            let (sin, cos) = theta.sin_cos();
            test.push(Vertex {
                x: 0.1 * theta * cos,
                y: 0.1 * theta * sin,
                _zero: 0,
                shape: Shape::Arrow,
                extra: f32::NAN,
                level: (i ^ j) as u8,
                alpha: 255,
            });
        }
        let _ = socket.send_to(&encoder.encode(), "127.0.0.1:12345");
        thread::sleep(Duration::from_millis(200));
    }
}
