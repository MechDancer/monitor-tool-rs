use super::Pose;
use iced::Color;

macro_rules! sizeof {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
}

/// 解码
pub mod input {
    use std::fmt::Display;

    use super::{
        super::{BorderMode, Pose},
        Config, PoseOrElse,
    };
    use iced::Color;

    /// 可从帧中解出话题的流
    pub struct TopicInputStream<'a> {
        title: &'a str,
        mode: BorderMode,
        buffer: &'a [u8],
    }

    /// 基于引用的话题缓存
    pub struct Topic<'a> {
        pub title: &'a str,
        pub mode: BorderMode,
        pub topic: &'a str,
        pub config: Config,
        pub items: ItemInputStream<'a>,
    }

    impl Display for Topic<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "title: {}, topic: {}, size: {}, flags: ",
                self.title,
                self.topic,
                self.config.size(),
            )?;
            let mut any = false;
            if self.config.object_mode() {
                any = true;
                f.write_str("object")?;
            }
            if self.config.clear() {
                if any {
                    f.write_str(" | ")?;
                } else {
                    any = true;
                }
                f.write_str("clear")?;
            }
            if self.config.display() {
                if any {
                    f.write_str(" | ")?;
                } else {
                    any = true;
                }
                f.write_str("display")?;
            }
            if self.config.display() {
                if any {
                    f.write_str(" | ")?;
                } else {
                    any = true;
                }
                f.write_str("display")?;
            }
            if self.config.connecting() {
                if any {
                    f.write_str(" | ")?;
                }
                f.write_str("connecting")?;
            }
            write!(f, ", size: {}", self.items.len())?;
            Ok(())
        }
    }

    impl<'a> TopicInputStream<'a> {
        /// 借用字节数组并构造一个话题流
        pub fn new(buffer: &'a [u8]) -> Self {
            let len = buffer[0] as usize;
            Self {
                title: unsafe { std::str::from_utf8_unchecked(&buffer[1..1 + len]) },
                mode: unsafe { *(buffer[1 + len..].as_ptr() as *const BorderMode) },
                buffer: &buffer[1 + len + sizeof!(BorderMode)..],
            }
        }

        /// 从流前分割一个字符串
        fn slice_str(&mut self) -> &'a str {
            let len = self.buffer[0] as usize;
            let end = 1 + len;
            let slice = &self.buffer[1..end];
            self.buffer = &self.buffer[end..];
            unsafe { std::str::from_utf8_unchecked(slice) }
        }

        /// 从流前分割一个位姿流
        fn slice_items(&mut self) -> &'a [PoseOrElse] {
            let len = unsafe { *(self.buffer.as_ptr() as *const u16) } as usize;
            let end = 2 + len * sizeof!(PoseOrElse);
            let ptr = (&self.buffer[2..]).as_ptr() as *const PoseOrElse;
            self.buffer = &self.buffer[end..];
            unsafe { std::slice::from_raw_parts(ptr, len) }
        }

        /// 从流前分割一个话题配置
        fn slice_config(&mut self) -> Config {
            let config = (&self.buffer).as_ptr() as *const Config;
            self.buffer = &self.buffer[sizeof!(Config)..];
            unsafe { *config }
        }
    }

    /// 可从话题解出位姿的流
    pub enum ItemInputStream<'a> {
        Objects(ObjectStream<'a>),
        Poses(PoseStream<'a>),
    }

    pub struct ObjectStream<'a>(&'a [PoseOrElse]);
    pub struct PoseStream<'a> {
        state: Color,
        items: &'a [PoseOrElse],
    }

    impl ItemInputStream<'_> {
        fn len(&self) -> usize {
            match self {
                ItemInputStream::Objects(stream) => stream.0.len(),
                ItemInputStream::Poses(stream) => stream.items.len(),
            }
        }
    }

    impl<'a> Iterator for TopicInputStream<'a> {
        type Item = Topic<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.buffer.is_empty() {
                None
            } else {
                let title = self.title;
                let mode = self.mode;
                let topic = self.slice_str();
                let config = self.slice_config();
                let objective = config.object_mode();
                let items = self.slice_items();
                Some(Self::Item {
                    title,
                    mode,
                    topic,
                    config,
                    items: if objective {
                        ItemInputStream::Objects(ObjectStream(items))
                    } else {
                        ItemInputStream::Poses(PoseStream {
                            state: Default::default(),
                            items,
                        })
                    },
                })
            }
        }
    }

    impl<'a> Iterator for ObjectStream<'a> {
        type Item = (&'a [Pose], Color);

        fn next(&mut self) -> Option<Self::Item> {
            if self.0.is_empty() || self.0[0].is_pose() {
                return None;
            }
            let color = self.0[0].color();
            for i in 1..self.0.len() {
                if !self.0[i].is_pose() {
                    let ptr = (&self.0[1..i]).as_ptr() as *const Pose;
                    self.0 = &self.0[i..];
                    return Some((unsafe { std::slice::from_raw_parts(ptr, i - 1) }, color));
                }
            }
            None
        }
    }

    impl<'a> Iterator for PoseStream<'a> {
        type Item = (Pose, Color);

        fn next(&mut self) -> Option<Self::Item> {
            while !self.items.is_empty() {
                let item = self.items[0];
                self.items = &self.items[1..];
                if item.is_pose() {
                    return Some((item.pose(), self.state));
                } else {
                    self.state = item.color();
                }
            }
            None
        }
    }
}

pub mod output {
    use super::{
        super::{BorderMode, Pose},
        Config, PoseOrElse,
    };
    use iced::Color;

    pub struct FrameOutputStream(Vec<u8>);

    pub struct PoseOutputStream<'a> {
        vec: &'a mut Vec<u8>,
        begin: usize,
    }

    impl FrameOutputStream {
        pub fn new(title: &str, mode: BorderMode) -> Self {
            let bytes = title.as_bytes();
            let mut buffer = vec![0u8; 1 + bytes.len() + sizeof!(BorderMode)];
            buffer[0] = bytes.len() as u8;
            buffer[1..bytes.len() + 1].copy_from_slice(bytes);
            unsafe { *(buffer[1 + bytes.len()..].as_ptr() as *mut BorderMode) = mode };
            Self(buffer)
        }

        pub fn push_topic<'a>(
            &'a mut self,
            topic: &str,
            config: Config,
            default_color: Color,
        ) -> PoseOutputStream<'a> {
            let bytes = topic.as_bytes();
            // push topic str
            self.0.push(bytes.len() as u8);
            self.0.extend_from_slice(bytes);
            // push config
            self.0.extend_from_slice(config.as_slice());
            // push size
            let begin = self.0.len();
            self.0.extend_from_slice(&[0u8; sizeof!(u16)]);
            let mut stream = PoseOutputStream {
                vec: &mut self.0,
                begin,
            };
            stream.push_color(default_color);
            stream
        }

        pub fn renew(&mut self) -> Vec<u8> {
            let mut other = Vec::from(&self.0[..1 + (self.0[0] as usize) + sizeof!(BorderMode)]);
            std::mem::swap(&mut self.0, &mut other);
            other
        }

        pub fn to_vec(self) -> Vec<u8> {
            self.0
        }
    }

    impl<'a> PoseOutputStream<'a> {
        fn push(&mut self, any: impl Into<PoseOrElse>) {
            unsafe {
                *(self.vec[self.begin..].as_mut_ptr() as *mut u16) += 1;
                self.vec.extend_from_slice(&any.into().slice);
            }
        }

        pub fn push_pose(&mut self, pose: Pose) {
            self.push(pose);
        }

        pub fn push_color(&mut self, color: Color) {
            self.push(color);
        }
    }
}

#[derive(Clone, Copy)]
struct RGBA(u8, u8, u8, u8);

#[derive(Clone, Copy)]
union PoseOrElse {
    pose: Pose,
    color: (f32, RGBA),
    slice: [u8; sizeof!(Pose)],
}

impl From<Pose> for PoseOrElse {
    fn from(pose: Pose) -> Self {
        Self { pose }
    }
}

impl From<Color> for PoseOrElse {
    fn from(color: Color) -> Self {
        Self {
            color: (
                f32::NAN,
                RGBA(
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                    (color.a * 255.0) as u8,
                ),
            ),
        }
    }
}

impl PoseOrElse {
    fn is_pose(&self) -> bool {
        !unsafe { self.pose }.x.is_nan()
    }

    fn pose(&self) -> Pose {
        unsafe { self.pose }
    }

    fn color(&self) -> Color {
        let RGBA(r, g, b, a) = unsafe { self.color }.1;
        Color::from_rgba8(r, g, b, a as f32 / 255.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Config {
    size_low: u16,
    size_high: u8,
    flags: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(1000, false, false, true, false, false)
    }
}

impl Config {
    fn new(
        size: usize,
        object_mode: bool,
        clear: bool,
        display: bool,
        focus: bool,
        connecting: bool,
    ) -> Self {
        let mut flags = 0u8;
        if object_mode {
            flags |= 0b00001;
        }
        if clear {
            flags |= 0b00010;
        }
        if display {
            flags |= 0b00100;
        }
        if focus {
            flags |= 0b01000;
        }
        if connecting {
            flags |= 0b10000;
        }

        Self {
            size_low: size as u16,
            size_high: (size >> 16) as u8,
            flags,
        }
    }

    pub fn size(&self) -> usize {
        ((self.size_high as usize) << 16) + self.size_low as usize
    }

    pub fn object_mode(&self) -> bool {
        self.flags & 0b00001 != 0
    }

    pub fn clear(&self) -> bool {
        self.flags & 0b00010 != 0
    }

    pub fn display(&self) -> bool {
        self.flags & 0b00100 != 0
    }

    pub fn focus(&self) -> bool {
        self.flags & 0b01000 != 0
    }

    pub fn connecting(&self) -> bool {
        self.flags & 0b10000 != 0
    }

    fn as_slice<'a>(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, sizeof!(Self)) }
    }
}

#[test]
fn verify_size() {
    assert_eq!(sizeof!(Config), 4);
    assert_eq!(sizeof!(PoseOrElse), sizeof!(Pose));
}

#[test]
fn test_encode_decode() {
    use super::{BorderMode, PolarAxis};
    use std::f32::consts::*;

    const TITLE: &str = "title";
    const MODE: BorderMode = BorderMode::Polar(PolarAxis::Top);
    const TOPIC: [&str; 2] = ["topic0", "topic1"];
    let configs: [Config; 2] = [
        Config::new(1000, false, true, true, false, false),
        Config::new(2000, false, false, true, false, true),
    ];
    const FRAME1: [Pose; 2] = [
        Pose {
            x: 1.0,
            y: 2.0,
            theta: PI,
        },
        Pose {
            x: 2.0,
            y: -1.0,
            theta: PI,
        },
    ];
    const FRAME2: [Pose; 3] = [
        Pose {
            x: 1.0,
            y: 0.0,
            theta: FRAC_PI_2,
        },
        Pose {
            x: 2.0,
            y: 0.0,
            theta: FRAC_PI_2,
        },
        Pose {
            x: 3.0,
            y: 0.0,
            theta: FRAC_PI_2,
        },
    ];

    let mut output = output::FrameOutputStream::new(TITLE, MODE);
    let mut topic = output.push_topic(TOPIC[0], configs[0], Color::BLACK);
    for pose in FRAME1 {
        topic.push_pose(pose);
    }
    let mut topic = output.push_topic(TOPIC[1], configs[1], Color::from_rgb8(255, 0, 0));
    for pose in FRAME2 {
        topic.push_pose(pose);
    }
    let buffer = output.to_vec();
    // --------------------------------------
    let input = input::TopicInputStream::new(buffer.as_slice());
    let mut i = 0;
    for topic in input {
        assert_eq!(topic.title, TITLE);
        assert_eq!(topic.mode, MODE);
        assert_eq!(topic.topic, TOPIC[i]);
        assert_eq!(topic.config, configs[i]);
        i += 1;
        println!("{}", topic);
        match topic.items {
            input::ItemInputStream::Objects(stream) => {
                for (pose, color) in stream {
                    println!("({:?}) {:?}", color, pose);
                }
            }
            input::ItemInputStream::Poses(stream) => {
                for (pose, color) in stream {
                    println!("({:?}) {:?}", color, pose);
                }
            }
        }
    }
}
