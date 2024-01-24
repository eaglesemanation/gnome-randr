use core::{
    cmp::{Eq, Ord},
    ffi::CStr,
    hash::Hash,
};
use std::sync::Arc;

use crate::DbusArg;

pub(crate) trait DbusDictKey: DbusArg<Self> + Hash + Eq + Ord {}

macro_rules! dbusdictkey_implement_primitives {
    ($ty:ty) => {
        impl<'a> DbusDictKey for $ty {}
    };
    ($ty:ty, $($tys:ty),+) => {
        dbusdictkey_implement_primitives!($ty);
        dbusdictkey_implement_primitives!($($tys),*);
    };
}

// I have no clue how dbus-rs has f64 and std::fs::File as a DictKey, they don't implement
// core::hash::Hash
dbusdictkey_implement_primitives!(
    u8,
    u16,
    u32,
    u64,
    i16,
    i32,
    i64,
    bool,
    String,
    &'a str,
    &'a CStr
);

impl<'a, T: DbusDictKey> DbusDictKey for &'a T {}
impl<T: DbusDictKey> DbusDictKey for Box<T> {}
impl<T: DbusDictKey + Clone> DbusDictKey for Arc<T> {}
