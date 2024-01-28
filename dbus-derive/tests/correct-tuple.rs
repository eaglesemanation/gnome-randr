use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};

use dbus::arg::Arg;
use dbus_derive::{DbusArgs, DbusEnum, DbusPropMap};

#[derive(DbusArgs, Default, Debug)]
pub struct ArgsNamed {
    pub arg_struct: NestedArg,
    pub arg_vec_struct: Vec<NestedArg>,
}

#[derive(DbusArgs, Default, Debug)]
pub struct ArgsNamedWrapper(pub NestedArg);

#[derive(DbusArgs, Default, Debug)]
pub struct ArgsUnnamed(pub NestedArg, pub Vec<NestedArg>);

#[derive(DbusArgs, Default, Debug)]
pub struct ArgsUnnamedWrapper(pub NestedArg);

#[derive(DbusArgs, Default, Debug)]
pub struct NestedArg {
    pub arg_i32: i32,
    pub arg_u32: u32,
    pub arg_string: String,
    //pub arg_str_ref: &'a str,
    //pub arg_slice: &'a [bool],
    pub arg_vec: Vec<f64>,
    pub arg_map: HashMap<i16, u16>,
    pub arg_tree: BTreeMap<i64, u64>,
    pub arg_enum: ChoiceArg,
    pub arg_props: PropsArg,
}

#[derive(DbusEnum, Debug, Default, Clone, Copy)]
#[dbus_enum(as_type = "u8")]
pub enum ChoiceArg {
    #[default]
    Choice0,
    Choice1,
}

impl From<ChoiceArg> for u8 {
    fn from(value: ChoiceArg) -> Self {
        match value {
            ChoiceArg::Choice0 => 0,
            ChoiceArg::Choice1 => 1,
        }
    }
}

impl TryFrom<u8> for ChoiceArg {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ChoiceArg::Choice0),
            1 => Ok(ChoiceArg::Choice1),
            _ => Err("Invalid ChoiceArg u8 representation"),
        }
    }
}

#[derive(DbusPropMap, Debug, Default, Clone)]
pub struct PropsArg {
    pub arg1: Option<String>,
    pub arg2: Option<u32>,
}

#[test]
fn conversion() -> Result<(), Box<dyn Error>> {
    let nested_sig = "(iusada{nq}a{xt}ya{sv})".to_string();
    let full_sig = format!("({nested_sig}a{nested_sig})");
    assert_eq!(nested_sig, NestedArg::signature().to_string());
    assert_eq!(nested_sig, ArgsNamedWrapper::signature().to_string());
    assert_eq!(nested_sig, ArgsUnnamedWrapper::signature().to_string());
    assert_eq!(full_sig, ArgsNamed::signature().to_string());
    assert_eq!(full_sig, ArgsUnnamed::signature().to_string());
    Ok(())
}
