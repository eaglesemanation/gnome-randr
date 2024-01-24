use core::{convert::Infallible, ffi::CStr};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    sync::Arc,
};

use crate::dict_key::DbusDictKey;

pub trait DbusArg<T>
where
    Self: Sized,
{
    type Error;

    fn dbus_arg_try_from(self) -> Result<T, Self::Error>;
    fn dbus_arg_try_into(value: T) -> Result<Self, Self::Error>;
}

macro_rules! dbusarg_implement_primitives {
    ($ty:ty) => {
        impl<'a> DbusArg<$ty> for $ty {
            // Both types are the same, no converion needed, therefore should not fail
            type Error = Infallible;

            fn dbus_arg_try_from(self) -> Result<$ty, Self::Error> {
                Ok(self)
            }
            fn dbus_arg_try_into(value: $ty) -> Result<Self, Self::Error> {
                Ok(value)
            }
        }
    };
    ($ty:ty, $($tys:ty),+) => {
        dbusarg_implement_primitives!($ty);
        dbusarg_implement_primitives!($($tys),*);
    };
}

dbusarg_implement_primitives!(
    u8,
    u16,
    u32,
    u64,
    i16,
    i32,
    i64,
    f64,
    bool,
    String,
    &'a str,
    &'a CStr,
    File
);

macro_rules! dbusarg_implement_with_generic {
    ($ty:ty) => {
        impl<'a, T> DbusArg<$ty> for $ty
        where
            T: DbusArg<T>,
        {
            // Both types are the same, no converion needed, therefore should not fail
            type Error = Infallible;

            fn dbus_arg_try_from(self) -> Result<$ty, Self::Error> {
                Ok(self)
            }
            fn dbus_arg_try_into(value: $ty) -> Result<Self, Self::Error> {
                Ok(value)
            }
        }
    };
    ($ty:ty, $($tys:ty),+) => {
        dbusarg_implement_with_generic!($ty);
        dbusarg_implement_with_generic!($($tys),*);
    };
}

dbusarg_implement_with_generic!(&'a T, Box<T>, Arc<T>, &'a [T]);

impl<Val, Arg> DbusArg<Vec<Val>> for Vec<Arg>
where
    Arg: DbusArg<Val>,
{
    type Error = <Arg as DbusArg<Val>>::Error;

    fn dbus_arg_try_from(self) -> Result<Vec<Val>, Self::Error> {
        self.into_iter().map(Arg::dbus_arg_try_from).collect()
    }
    fn dbus_arg_try_into(value: Vec<Val>) -> Result<Self, Self::Error> {
        value.into_iter().map(Arg::dbus_arg_try_into).collect()
    }
}

impl<Val, Arg, const N: usize> DbusArg<[Val; N]> for [Arg; N]
where
    Arg: DbusArg<Val>,
    Val: Sized,
{
    type Error = <Arg as DbusArg<Val>>::Error;

    fn dbus_arg_try_from(self) -> Result<[Val; N], Self::Error> {
        self.try_map(Arg::dbus_arg_try_from)
    }
    fn dbus_arg_try_into(value: [Val; N]) -> Result<Self, Self::Error> {
        value.try_map(Arg::dbus_arg_try_into)
    }
}

impl<Val, Arg, K> DbusArg<HashMap<K, Val>> for HashMap<K, Arg>
where
    Arg: DbusArg<Val>,
    K: DbusDictKey,
{
    type Error = <Arg as DbusArg<Val>>::Error;

    fn dbus_arg_try_from(self) -> Result<HashMap<K, Val>, Self::Error> {
        let mut res = HashMap::new();
        for (k, v) in self {
            res.insert(k, Arg::dbus_arg_try_from(v)?);
        }
        Ok(res)
    }
    fn dbus_arg_try_into(value: HashMap<K, Val>) -> Result<Self, Self::Error> {
        let mut res = HashMap::new();
        for (k, v) in value {
            res.insert(k, Arg::dbus_arg_try_into(v)?);
        }
        Ok(res)
    }
}

impl<Val, Arg, K> DbusArg<BTreeMap<K, Val>> for BTreeMap<K, Arg>
where
    Arg: DbusArg<Val>,
    K: DbusDictKey,
{
    type Error = <Arg as DbusArg<Val>>::Error;

    fn dbus_arg_try_from(self) -> Result<BTreeMap<K, Val>, Self::Error> {
        let mut res = BTreeMap::new();
        for (k, v) in self {
            res.insert(k, Arg::dbus_arg_try_from(v)?);
        }
        Ok(res)
    }
    fn dbus_arg_try_into(value: BTreeMap<K, Val>) -> Result<Self, Self::Error> {
        let mut res = BTreeMap::new();
        for (k, v) in value {
            res.insert(k, Arg::dbus_arg_try_into(v)?);
        }
        Ok(res)
    }
}

impl<T: dbus::arg::RefArg> DbusArg<dbus::arg::Variant<T>> for dbus::arg::Variant<T> {
    // Both types are the same, no converion needed, therefore should not fail
    type Error = Infallible;

    fn dbus_arg_try_from(self) -> Result<dbus::arg::Variant<T>, Self::Error> {
        Ok(self)
    }
    fn dbus_arg_try_into(value: dbus::arg::Variant<T>) -> Result<Self, Self::Error> {
        Ok(value)
    }
}
