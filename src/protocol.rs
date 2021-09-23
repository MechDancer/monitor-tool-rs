macro_rules! sizeof {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
}

/// 解码
pub mod input {
    use super::{super::BorderMode, ItemHeader};

    pub struct Graph<'a> {
        title: &'a str,
        border: BorderMode,
        stream: TopicInputStream<'a>,
    }

    pub struct Topic<'a> {
        topic: &'a str,
        stream: ItemInputStream<'a>,
    }

    pub trait Item {}

    struct Stream<'a>(&'a [u8]);
    pub struct GraphInputStream<'a>(Stream<'a>);
    pub struct TopicInputStream<'a>(Stream<'a>);
    pub struct ItemInputStream<'a>(Stream<'a>);

    impl<'a> Stream<'a> {
        fn save_and_jump(&mut self, len: usize) -> *const u8 {
            let ptr = self.0.as_ptr();
            self.0 = &self.0[len..];
            ptr
        }

        fn split(&mut self, len: usize) -> &'a [u8] {
            let slice = &self.0[..len];
            self.0 = &self.0[len..];
            slice
        }

        fn read<T: Copy>(&mut self) -> T {
            let ptr = self.save_and_jump(sizeof!(T)) as *const T;
            unsafe { *ptr }
        }

        fn read_slice<T: Sized>(&mut self, len: usize) -> &'a [T] {
            let ptr = self.save_and_jump(sizeof!(T) * len) as *const T;
            unsafe { std::slice::from_raw_parts(ptr, len) }
        }

        fn read_str(&mut self) -> &'a str {
            let len = self.read::<u8>() as usize;
            let slice = self.split(len);
            unsafe { std::str::from_utf8_unchecked(slice) }
        }
    }

    impl<'a> From<&'a [u8]> for Graph<'a> {
        fn from(slice: &'a [u8]) -> Self {
            let mut stream = Stream(slice);
            Self {
                title: stream.read_str(),
                border: stream.read::<BorderMode>(),
                stream: TopicInputStream(stream),
            }
        }
    }

    impl<'a> From<&'a [u8]> for Topic<'a> {
        fn from(slice: &'a [u8]) -> Self {
            let mut stream = Stream(slice);
            Self {
                topic: stream.read_str(),
                stream: ItemInputStream(stream),
            }
        }
    }

    impl<'a> Iterator for GraphInputStream<'a> {
        type Item = Graph<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.0 .0.is_empty() {
                None
            } else {
                let len = self.0.read::<u32>() as usize;
                let slice = self.0.split(len);
                Some(Graph::from(slice))
            }
        }
    }

    impl<'a> Iterator for TopicInputStream<'a> {
        type Item = Topic<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.0 .0.is_empty() {
                None
            } else {
                let len = self.0.read::<u16>() as usize;
                let slice = self.0.split(len);
                Some(Topic::from(slice))
            }
        }
    }

    impl<'a> Iterator for ItemInputStream<'a> {
        type Item = ItemHeader;

        fn next(&mut self) -> Option<Self::Item> {
            todo!()
        }
    }
}

pub mod output {
    //     use super::{
    //         super::{BorderMode, Pose},
    //         Config, PoseOrElse,
    //     };
    //     use iced::Color;

    //     pub struct FrameOutputStream(Vec<u8>);

    //     pub struct PoseOutputStream<'a> {
    //         vec: &'a mut Vec<u8>,
    //         begin: usize,
    //     }

    //     impl FrameOutputStream {
    //         pub fn new(title: &str, mode: BorderMode) -> Self {
    //             let bytes = title.as_bytes();
    //             let mut buffer = vec![0u8; 1 + bytes.len() + sizeof!(BorderMode)];
    //             buffer[0] = bytes.len() as u8;
    //             buffer[1..bytes.len() + 1].copy_from_slice(bytes);
    //             unsafe { *(buffer[1 + bytes.len()..].as_ptr() as *mut BorderMode) = mode };
    //             Self(buffer)
    //         }

    //         pub fn push_topic<'a>(
    //             &'a mut self,
    //             topic: &str,
    //             config: Config,
    //             default_color: Color,
    //         ) -> PoseOutputStream<'a> {
    //             let bytes = topic.as_bytes();
    //             // push topic str
    //             self.0.push(bytes.len() as u8);
    //             self.0.extend_from_slice(bytes);
    //             // push config
    //             self.0.extend_from_slice(config.as_slice());
    //             // push size
    //             let begin = self.0.len();
    //             self.0.extend_from_slice(&[0u8; sizeof!(u16)]);
    //             let mut stream = PoseOutputStream {
    //                 vec: &mut self.0,
    //                 begin,
    //             };
    //             stream.push_color(default_color);
    //             stream
    //         }

    //         pub fn renew(&mut self) -> Vec<u8> {
    //             let mut other = Vec::from(&self.0[..1 + (self.0[0] as usize) + sizeof!(BorderMode)]);
    //             std::mem::swap(&mut self.0, &mut other);
    //             other
    //         }

    //         pub fn to_vec(self) -> Vec<u8> {
    //             self.0
    //         }
    //     }

    //     impl<'a> PoseOutputStream<'a> {
    //         fn push(&mut self, any: impl Into<PoseOrElse>) {
    //             unsafe {
    //                 *(self.vec[self.begin..].as_mut_ptr() as *mut u16) += 1;
    //                 self.vec.extend_from_slice(&any.into().slice);
    //             }
    //         }

    //         pub fn push_pose(&mut self, pose: Pose) {
    //             self.push(pose);
    //         }

    //         pub fn push_color(&mut self, color: Color) {
    //             self.push(color);
    //         }
    //     }
}

#[derive(Clone, Copy)]
pub enum ControlType {
    ClearAll,
    ForLevel { level: u8 },
    ForPath { connecting: bool },
    Color { level: u8 },
}

#[derive(Clone, Copy)]
pub enum ItemHeader {
    Control(ControlType),
    Path { level: u8, size: u16 },
    Polyline { level: u8, size: u16 },
    Polygon { level: u8, size: u16 },
    Circle { level: u8 },
}

#[test]
fn verify_size() {
    assert_eq!(sizeof!(ControlType), 2);
    assert_eq!(sizeof!(ItemHeader), 4);
}
