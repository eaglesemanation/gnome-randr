#![feature(array_try_map)]

use core::convert::Infallible;

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

dbusarg_implement_primitives!(u8, u16, u32, u64, i16, i32, i64, f64, bool, String, &'a str);

impl<'a, T> DbusArg<&'a T> for &'a T
where
    T: DbusArg<T>,
{
    // Both types are the same, no converion needed, therefore should not fail
    type Error = Infallible;

    fn dbus_arg_try_from(self) -> Result<&'a T, Self::Error> {
        Ok(self)
    }

    fn dbus_arg_try_into(value: &'a T) -> Result<Self, Self::Error> {
        Ok(value)
    }
}

impl<'a, T> DbusArg<&'a [T]> for &'a [T]
where
    T: DbusArg<T>,
{
    // Both types are the same, no converion needed, therefore should not fail
    type Error = Infallible;

    fn dbus_arg_try_from(self) -> Result<&'a [T], Self::Error> {
        Ok(self)
    }

    fn dbus_arg_try_into(value: &'a [T]) -> Result<Self, Self::Error> {
        Ok(value)
    }
}

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
