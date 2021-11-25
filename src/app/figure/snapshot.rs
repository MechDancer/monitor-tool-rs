use crate::Vertex;

use super::content::TopicBuffer;
use async_std::{fs::File, io::WriteExt, path::PathBuf};
use palette::{rgb::channels::Argb, Packed, Srgba};
use std::collections::HashMap;

pub struct FigureSnapshot(pub(crate) HashMap<String, TopicBuffer>);

macro_rules! write_async {
    (     $bytes:expr => $file:expr) => {
        $file.write_all($bytes).await
    };
    (str; $str:expr   => $file:expr) => {
        write_async!($str.as_bytes() => $file)
    };
}

impl FigureSnapshot {
    pub async fn save(self, path: PathBuf) -> std::io::Result<()> {
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
            for (_, v) in buffer.queue {
                let Vertex {
                    x,
                    y,
                    level,
                    alpha,
                    _zero: _,
                    shape,
                    extra,
                } = v;
                write_async!(str; format!("{:03} {:03}|{} {}|{} {}\n", level, alpha, x, y, shape, extra) => file)?;
            }
            write_async!(&[b'\n'] => file)?;
        }
        Ok(())
    }

    pub async fn load(path: PathBuf) -> std::io::Result<Self> {
        let _stream = File::open(path).await?;
        let mut result = Default::default();
        Ok(Self(result))
    }
}
