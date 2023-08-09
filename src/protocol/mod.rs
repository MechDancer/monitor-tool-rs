#[cfg(feature = "app")]
mod decode;

#[cfg(feature = "app")]
pub(crate) use decode::decode;

#[cfg(feature = "client")]
mod encode;

#[cfg(feature = "client")]
pub use encode::*;

/// 图层是否显示
#[derive(Clone, Copy)]
enum Visibility {
    NothingToDo = 0,
    Visible = 0x55,
    Invisible = 0xaa,
}

impl Default for Visibility {
    #[inline]
    fn default() -> Self {
        Self::NothingToDo
    }
}

impl From<Option<bool>> for Visibility {
    #[inline]
    fn from(v: Option<bool>) -> Self {
        match v {
            None => Visibility::NothingToDo,
            Some(true) => Visibility::Visible,
            Some(false) => Visibility::Invisible,
        }
    }
}
