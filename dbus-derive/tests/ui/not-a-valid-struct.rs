use dbus_derive::DbusArgs;

#[derive(DbusArgs)]
pub enum Arg {
    Opt1,
    Opt2,
}

#[derive(DbusArgs)]
pub union Arg2 {
    opt1: i32,
    opt2: u32,
}

#[derive(DbusArgs)]
pub struct Arg3;

fn main() {}
