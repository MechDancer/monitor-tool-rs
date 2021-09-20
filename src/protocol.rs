use super::Pose;
use iced::Color;

pub struct FrameInputStream<'a> {
    title: &'a str,
    buffer: &'a [u8],
}

pub struct PoseInputStream<'a> {
    state: Color,
    buffer: &'a [u8],
}

pub struct Frame<'a> {
    title: &'a str,
    topic: &'a str,
    pose: &'a Pose,
    size: usize,
    connecting: bool,
    clear: bool,
    poses: PoseInputStream<'a>,
}

#[derive(Clone, Copy)]
struct RGBA(u8, u8, u8, u8);

#[derive(Clone, Copy)]
union PoseOrElse {
    pose: Pose,
    color: (f32, RGBA),
}

#[repr(C)]
struct Config {
    flags: u8,
    size: [u8; 3],
    pose: Pose,
}

macro_rules! sizeof {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
}

impl<'a> FrameInputStream<'a> {
    /// 从字节数组构造一个帧流
    fn new(buffer: &'a [u8]) -> Self {
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
    type Item = Frame<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            None
        } else {
            let topic = self.slice_str();
            let config = self.slice_config();
            let stream = self.slice_poses();
            Some(Self::Item {
                title: self.title,
                topic,
                pose: &config.pose,
                clear: config.flags & 1 != 0,
                connecting: config.flags & 2 != 0,
                size: 0,
                poses: stream,
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

#[test]
fn verify_size() {
    assert_eq!(sizeof!(Config), 16);
    assert_eq!(sizeof!(PoseOrElse), sizeof!(Pose));
}
