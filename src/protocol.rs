#[cfg(feature = "app")]
mod decode;

#[cfg(feature = "app")]
pub(crate) use decode::decode;

#[cfg(feature = "sender")]
mod packet;

#[cfg(feature = "sender")]
pub use packet::*;

/// 图层是否显示
enum Visible {
    NothingToDo = 0,
    Visible = 0x55,
    Invisible = 0xaa,
}

impl Default for Visible {
    #[inline]
    fn default() -> Self {
        Self::NothingToDo
    }
}

impl From<Option<bool>> for Visible {
    #[inline]
    fn from(v: Option<bool>) -> Self {
        match v {
            None => Visible::NothingToDo,
            Some(true) => Visible::Visible,
            Some(false) => Visible::Invisible,
        }
    }
}
