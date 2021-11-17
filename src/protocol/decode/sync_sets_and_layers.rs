use crate::protocol::Visible;
use std::time::Duration;

#[derive(Clone, Copy)]
pub(super) struct SyncSets<'a>(pub &'a [u16], pub &'a [u8]);

#[derive(Clone, Copy)]
pub(super) struct Layers<'a>(pub &'a [u16], pub &'a [u8]);

macro_rules! size_of {
    ($ty:ty) => {
        std::mem::size_of::<$ty>()
    };
}

macro_rules! get_item_slice {
    ($obj:expr, $i:expr) => {{
        let begin = if $i == 0 { 0 } else { $obj.0[$i - 1] as usize };
        &$obj.1[begin..$obj.0[$i] as usize]
    }};
}

impl<'a> SyncSets<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, i: usize) -> (&'a str, &'a Duration) {
        let slice = get_item_slice!(self, i);
        unsafe {
            (
                std::str::from_utf8_unchecked(&slice[size_of!(Duration)..]),
                &*(slice.as_ptr() as *const Duration),
            )
        }
    }
}

impl<'a> Layers<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, i: usize) -> (&'a str, &'a Visible) {
        let slice = get_item_slice!(self, i);
        unsafe {
            (
                std::str::from_utf8_unchecked(&slice[size_of!(Visible)..]),
                &*(slice.as_ptr() as *const Visible),
            )
        }
    }
}
