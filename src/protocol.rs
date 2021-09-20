use super::Pose;

macro_rules! sizeof {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
}

#[derive(Clone, Copy)]
struct RGBA(u8, u8, u8, u8);

#[derive(Clone, Copy)]
union PoseOrElse {
    pose: Pose,
    color: (f32, RGBA),
    slice: [u8; sizeof!(Pose)],
}

#[repr(C)]
struct Config {
    size_low: u16,
    size_high: u8,
    flags: u8,
    pose: Pose,
}

impl Config {
    fn new(pose: &Pose, size: usize, connecting: bool, clear: bool) -> Self {
        let mut flags = 0u8;
        if connecting {
            flags &= 1;
        }
        if clear {
            flags &= 2;
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

    fn connecting(&self) -> bool {
        self.flags & 1 != 0
    }

    fn clear(&self) -> bool {
        self.flags & 2 != 0
    }

    fn as_slice<'a>(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(std::ptr::addr_of!(self) as *const u8, sizeof!(Self)) }
    }
}

pub mod input {
    use super::{super::Pose, Config, PoseOrElse, RGBA};
    use iced::Color;

    pub struct FrameInputStream<'a> {
        title: &'a str,
        buffer: &'a [u8],
    }

    pub struct PoseInputStream<'a> {
        state: Color,
        buffer: &'a [u8],
    }

    pub struct Topic<'a> {
        title: &'a str,
        topic: &'a str,
        pose: Pose,
        size: usize,
        connecting: bool,
        clear: bool,
        poses: PoseInputStream<'a>,
    }

    impl<'a> FrameInputStream<'a> {
        /// 从字节数组构造一个帧流
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

    impl<'a> Iterator for FrameInputStream<'a> {
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
                    connecting: config.connecting(),
                    clear: config.clear(),
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
}

pub mod output {
    use super::{super::Pose, Config, PoseOrElse, RGBA};
    use iced::Color;

    pub struct FrameOutputStream {
        buffer: Vec<u8>,
        topics: Vec<usize>,
    }

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
            Self {
                buffer,
                topics: Vec::new(),
            }
        }

        pub fn push_topic<'a>(
            &'a mut self,
            topic: &str,
            pose: &Pose,
            size: usize,
            connecting: bool,
            clear: bool,
        ) -> PoseOutputStream<'a> {
            let bytes = topic.as_bytes();
            // push topic str
            self.topics.push(self.buffer.len());
            self.buffer.push(bytes.len() as u8);
            self.buffer.extend_from_slice(bytes);
            // push config
            self.buffer
                .extend_from_slice(Config::new(pose, size, connecting, clear).as_slice());
            // push size
            let begin = self.buffer.len();
            self.buffer.extend_from_slice(&[0u8; sizeof!(u16)]);
            PoseOutputStream {
                vec: &mut self.buffer,
                begin,
            }
        }
    }

    impl<'a> PoseOutputStream<'a> {
        unsafe fn push(&mut self, any: PoseOrElse) {
            *(self.vec[self.begin..].as_mut_ptr() as *mut u16) += 1;
            self.vec.extend_from_slice(&any.slice);
        }

        pub fn push_pose(&mut self, pose: Pose) {
            unsafe { self.push(PoseOrElse { pose }) }
        }

        pub fn push_color(&mut self, color: Color) {
            unsafe {
                self.push(PoseOrElse {
                    color: (
                        f32::NAN,
                        RGBA(
                            (color.r * 255.0) as u8,
                            (color.g * 255.0) as u8,
                            (color.b * 255.0) as u8,
                            (color.a * 255.0) as u8,
                        ),
                    ),
                })
            }
        }
    }
}

#[test]
fn verify_size() {
    assert_eq!(sizeof!(Config), 16);
    assert_eq!(sizeof!(PoseOrElse), sizeof!(Pose));
}
