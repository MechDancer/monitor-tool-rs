use super::Pose;

pub struct FrameInputStream<'a> {
    title: &'a str,
    buffer: &'a [u8],
}

pub struct PoseInputStream<'a>(&'a [u8]);

pub struct Frame<'a> {
    title: &'a str,
    topic: &'a str,
    pose: &'a Pose,
    size: usize,
    connecting: bool,
    clear: bool,
    poses: PoseInputStream<'a>,
}

struct RGBA(u8, u8, u8, u8);

#[repr(C)]
struct Config {
    flags: u8,
    size: [u8; 3],
    pose: Pose,
}

impl<'a> FrameInputStream<'a> {
    fn from(buffer: &'a [u8]) -> Self {
        let mut result = Self { title: "", buffer };
        result.title = result.slice_str();
        result
    }

    fn slice_str(&mut self) -> &'a str {
        let len = self.buffer[0] as usize;
        let str = unsafe { std::str::from_utf8_unchecked(&self.buffer[1..1 + len]) };
        self.buffer = &self.buffer[1 + len..];
        str
    }

    fn slice_config(&mut self) -> &'a Config {
        let config = (&self.buffer).as_ptr() as *const Config;
        self.buffer = &self.buffer[std::mem::size_of::<Config>()..];
        unsafe { config.as_ref().unwrap() }
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
            let stream = PoseInputStream(&self.buffer);
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

#[test]
fn verify_size() {
    assert_eq!(std::mem::size_of::<Config>(), 16);
}
