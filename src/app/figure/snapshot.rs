use super::content::TopicBuffer;
use crate::Vertex;
use async_std::{
    fs::{create_dir_all, File},
    io::{BufReader, WriteExt},
    path::PathBuf,
};
use iced::{futures::AsyncBufReadExt, Point};
use palette::{rgb::channels::Argb, Packed, Pixel, Srgba};
use std::{collections::HashMap, time::Instant};

pub struct FigureSnapshot(pub(crate) HashMap<String, TopicBuffer>);

macro_rules! write_async {
    (     $bytes:expr => $file:expr) => {
        $file.write_all($bytes).await
    };
    (str; $str:expr   => $file:expr) => {
        write_async!($str.as_bytes() => $file)
    };
}

macro_rules! read_line {
    ($reader:expr => $line:expr) => {{
        $line.clear();
        match $reader.read_line(&mut $line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => $line.trim(),
        }
    }};
}

macro_rules! some_or_break {
    ($option:expr) => {{
        if let Some(x) = $option {
            x
        } else {
            break;
        }
    }};
}

impl FigureSnapshot {
    pub async fn save(self, path: PathBuf) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            create_dir_all(dir).await?;
        }
        let mut file = File::create(path).await?;
        for (topic, buffer) in self.0 {
            // 名字
            write_async!(str; format!("{}\n", topic) => file)?;
            // 颜色
            write_async!(str; format!("colors[{}]\n", buffer.color_map.len()) => file)?;
            for level in 0u8..255 {
                if let Some(color) = buffer.color_map.get(&level) {
                    let color = Srgba::new(color.r, color.g, color.b, color.a);
                    let color = Packed::<Argb>::from(color.into_format()).color;
                    write_async!(str; format!("{:03}|{:#08x}\n", level, color) => file)?;
                }
            }
            // 数据
            write_async!(str; format!("items[{}/{}]\n", buffer.queue.len(), buffer.capacity) => file)?;
            for (_, v) in buffer.queue.iter().rev() {
                let Vertex {
                    x,
                    y,
                    level,
                    alpha,
                    _zero: _,
                    shape,
                    extra,
                } = v;
                let alpha = *alpha as f32 / 2.55;
                let bytes = unsafe { *(v as *const _ as *const u128) };
                write_async!(str; format!("{:03}|{:10.3} {:10.3}|{} {:7.3}|{:3.0}% /{:032x}\n",
                                           level, x, y, shape, extra, alpha, bytes) => file)?;
            }
            // 空一行
            write_async!(&[b'\n'] => file)?;
        }
        Ok(())
    }

    pub async fn load(path: PathBuf) -> std::io::Result<(Point, Self)> {
        let file = File::open(path).await?;
        let mut reader = BufReader::new(file);

        let mut result = HashMap::<String, TopicBuffer>::new();
        let mut line = String::new();
        let now = Instant::now();
        let mut cx: f32 = 0.0;
        let mut cy: f32 = 0.0;
        let mut cn: usize = 0;
        loop {
            let topic = result.entry(read_line!(reader => line).into()).or_default();
            {
                let str = read_line!(reader => line);
                let str = str.trim_start_matches("colors[");
                let str = str.trim_end_matches("]");
                let len: usize = some_or_break!(str.parse().ok());
                for _ in 0..len {
                    let mut str = read_line!(reader => line).split('|');
                    let level = some_or_break!(str.next().and_then(|s| s.parse::<u8>().ok()));
                    let color: Option<[f32; 4]> = str
                        .next()
                        .and_then(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                        .map(|c| Srgba::from_u32::<Argb>(c).into_format().into_raw());
                    topic.color_map.insert(level, some_or_break!(color).into());
                }
            }
            {
                let str = read_line!(reader => line);
                let str = str.trim_start_matches("items[");
                let str = str.trim_end_matches("]");
                let mut str = str.split('/');
                let len: usize = some_or_break!(str.next().and_then(|s| s.parse().ok()));
                topic.capacity = some_or_break!(str.next().and_then(|s| s.parse().ok()));
                topic.queue.reserve(len);
                for _ in 0..len {
                    let data = read_line!(reader => line)
                        .split('/')
                        .last()
                        .and_then(|s| u128::from_str_radix(s, 16).ok());
                    let data = some_or_break!(data);
                    let data = unsafe { &*(&data as *const _ as *const Vertex) };
                    topic.queue.push_front((now, *data));
                    cx += data.x;
                    cy += data.y;
                    cn += 1;
                }
            }
            read_line!(reader => line);
        }
        Ok((
            Point {
                x: cx / cn as f32,
                y: cy / cn as f32,
            },
            Self(result),
        ))
    }
}
