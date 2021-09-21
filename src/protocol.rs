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

    use super::{super::Pose, Config, PoseOrElse};
    use iced::Color;

    /// 可从帧中解出话题的流
    pub struct TopicInputStream<'a> {
        title: &'a str,
        buffer: &'a [u8],
    }

    /// 基于引用的话题缓存
    pub struct Topic<'a> {
        pub title: &'a str,
        pub topic: &'a str,
        pub pose: Pose,
        pub size: usize,
        pub clear: bool,
        pub display: bool,
        pub connecting: bool,
        pub poses: PoseInputStream<'a>,
    }

    impl Display for Topic<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "title: {}, topic: {}, pose: {:?}, size: {}, flags: ",
                self.title, self.topic, self.pose, self.size,
            )?;
            let mut any = false;
            if self.clear {
                any = true;
                f.write_str("clear")?;
            }
            if self.display {
                if any {
                    f.write_str(" | ")?;
                } else {
                    any = true;
                }
                f.write_str("display")?;
            }
            if self.connecting {
                if any {
                    f.write_str(" | ")?;
                }
                f.write_str("connecting")?;
            }
            write!(f, ", size: {}", self.poses.len())?;
            Ok(())
        }
    }

    /// 可从话题解出位姿的流
    #[derive(Debug)]
    pub struct PoseInputStream<'a> {
        state: Color,
        buffer: &'a [u8],
    }

    impl<'a> TopicInputStream<'a> {
        /// 借用字节数组并构造一个话题流
        pub fn new(buffer: &'a [u8]) -> Self {
            let mut result = Self { title: "", buffer };
            result.title = result.slice_str();
            result
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
        fn slice_poses(&mut self) -> PoseInputStream<'a> {
            let len = unsafe { *(self.buffer.as_ptr() as *const u16) } as usize;
            let end = 2 + len * sizeof!(PoseOrElse);
            let slice = &self.buffer[2..end];
            self.buffer = &self.buffer[end..];
            PoseInputStream {
                state: Default::default(),
                buffer: slice,
            }
        }

        /// 从流前分割一个话题配置
        fn slice_config(&mut self) -> &'a Config {
            let config = (&self.buffer).as_ptr() as *const Config;
            self.buffer = &self.buffer[sizeof!(Config)..];
            unsafe { config.as_ref() }.unwrap()
        }
    }

    impl PoseInputStream<'_> {
        /// 剩余项数
        fn len(&self) -> usize {
            self.buffer.len() / sizeof!(PoseOrElse)
        }
    }

    impl<'a> Iterator for TopicInputStream<'a> {
        type Item = Topic<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.buffer.is_empty() {
                None
            } else {
                let topic = self.slice_str();
                let config = self.slice_config();
                Some(Self::Item {
                    title: self.title,
                    topic,
                    pose: config.pose(),
                    clear: config.clear(),
                    display: config.display(),
                    connecting: config.connecting(),
                    size: config.size(),
                    poses: self.slice_poses(),
                })
            }
        }
    }

    impl<'a> Iterator for PoseInputStream<'a> {
        type Item = (Pose, Color);

        fn next(&mut self) -> Option<Self::Item> {
            while !self.buffer.is_empty() {
                let pose_or_else = unsafe { *(self.buffer.as_ptr() as *const PoseOrElse) };
                self.buffer = &self.buffer[sizeof!(PoseOrElse)..];
                if pose_or_else.is_pose() {
                    return Some((pose_or_else.pose(), self.state));
                } else {
                    self.state = pose_or_else.color()
                }
            }
            None
        }
    }
}

pub mod output {
    use super::{super::Pose, Config, PoseOrElse};
    use iced::Color;

    pub struct FrameOutputStream(Vec<u8>);

    pub struct PoseOutputStream<'a> {
        vec: &'a mut Vec<u8>,
        begin: usize,
    }

    impl FrameOutputStream {
        pub fn new(title: &str) -> Self {
            let bytes = title.as_bytes();
            let mut buffer = vec![0u8; 1 + bytes.len()];
            buffer[0] = bytes.len() as u8;
            buffer[1..].copy_from_slice(bytes);
            Self(buffer)
        }

        pub fn push_topic<'a>(
            &'a mut self,
            topic: &str,
            pose: &Pose,
            size: usize,
            clear: bool,
            display: bool,
            connecting: bool,
            default_color: Color,
        ) -> PoseOutputStream<'a> {
            let bytes = topic.as_bytes();
            // push topic str
            self.0.push(bytes.len() as u8);
            self.0.extend_from_slice(bytes);
            // push config
            let config = Config::new(pose, size, clear, display, connecting);
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

#[repr(C)]
struct Config {
    size_low: u16,
    size_high: u8,
    flags: u8,
    pose: Pose,
}

impl Config {
    fn new(pose: &Pose, size: usize, clear: bool, display: bool, connecting: bool) -> Self {
        let mut flags = 0u8;
        if clear {
            flags |= 0b001;
        }
        if display {
            flags |= 0b010;
        }
        if connecting {
            flags |= 0b100;
        }

        Self {
            size_low: size as u16,
            size_high: (size >> 16) as u8,
            flags,
            pose: *pose,
        }
    }

    fn pose(&self) -> Pose {
        self.pose
    }

    fn size(&self) -> usize {
        ((self.size_high as usize) << 16) + self.size_low as usize
    }

    fn clear(&self) -> bool {
        self.flags & 0b001 != 0
    }

    fn display(&self) -> bool {
        self.flags & 0b010 != 0
    }

    fn connecting(&self) -> bool {
        self.flags & 0b100 != 0
    }

    fn as_slice<'a>(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, sizeof!(Self)) }
    }
}

#[test]
fn verify_size() {
    assert_eq!(sizeof!(Config), 16);
    assert_eq!(sizeof!(PoseOrElse), sizeof!(Pose));
}

#[test]
fn test_encode_decode() {
    use std::f32::consts::*;

    const TITLE: &str = "title";
    const TOPIC: [&str; 2] = ["topic0", "topic1"];

    let mut output = output::FrameOutputStream::new(TITLE);
    let mut topic0 = output.push_topic(
        TOPIC[0],
        &Pose {
            x: 1.0,
            y: 2.0,
            theta: PI,
        },
        1000,
        true,
        true,
        false,
        Color::BLACK,
    );
    topic0.push_pose(Pose {
        x: 2.0,
        y: -1.0,
        theta: FRAC_PI_2,
    });
    let mut topic1 = output.push_topic(
        TOPIC[1],
        &Pose {
            x: -1.0,
            y: -2.0,
            theta: PI,
        },
        2000,
        false,
        true,
        true,
        Color::from_rgb8(255, 0, 0),
    );
    topic1.push_pose(Pose {
        x: 1.0,
        y: 0.0,
        theta: FRAC_PI_2,
    });
    topic1.push_pose(Pose {
        x: 2.0,
        y: 0.0,
        theta: FRAC_PI_2,
    });
    topic1.push_pose(Pose {
        x: 3.0,
        y: 0.0,
        theta: FRAC_PI_2,
    });
    let buffer = output.to_vec();
    // --------------------------------------
    let input = input::TopicInputStream::new(buffer.as_slice());
    let mut i = 0;
    for topic in input {
        assert_eq!(topic.title, TITLE);
        assert_eq!(topic.topic, TOPIC[i]);
        i += 1;
        println!("{}", topic);
        for (pose, color) in topic.poses {
            println!("({:?}) {:?}", color, pose);
        }
    }
}
