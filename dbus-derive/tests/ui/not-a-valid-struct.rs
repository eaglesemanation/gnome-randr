use dbus_derive::DbusArgs;

#[derive(DbusArgs)]
pub enum Arg {
    Opt1,
    Opt2,
}

#[derive(DbusArgs)]
pub struct Arg3;

fn main() {}
