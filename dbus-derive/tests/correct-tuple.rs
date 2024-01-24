use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};

use dbus_derive::DbusArgs;
use dbus_traits::DbusArg;

#[derive(DbusArgs, Default, Debug)]
pub struct ArgsNamed<'a> {
    #[dbus_arg(derived)]
    pub arg_struct: NestedArg<'a>,
    #[dbus_arg(derived)]
    pub arg_vec_struct: Vec<NestedArg<'a>>,
}

#[derive(DbusArgs, Default, Debug)]
pub struct ArgsUnnamed<'a>(
    #[dbus_arg(derived)] pub NestedArg<'a>,
    #[dbus_arg(derived)] pub Vec<NestedArg<'a>>,
);

#[derive(DbusArgs, Default, Debug)]
pub struct NestedArg<'a> {
    pub arg_i32: i32,
    pub arg_u32: u32,
    pub arg_string: String,
    pub arg_str_ref: &'a str,
    pub arg_slice: &'a [bool],
    pub arg_array: [u8; 4],
    pub arg_vec: Vec<f64>,
    pub arg_map: HashMap<i16, u16>,
    pub art_tree: BTreeMap<i64, u64>,
    #[dbus_arg(target_type = "u8")]
    pub arg_enum: ChoiceArg,
    pub arg_props: dbus::arg::PropMap,
}

#[derive(Debug, Default)]
pub enum ChoiceArg {
    #[default]
    Choice0,
    Choice1,
}

impl DbusArg<u8> for ChoiceArg {
    type Error = &'static str;

    fn dbus_arg_try_from(self) -> Result<u8, Self::Error> {
        Ok(match self {
            ChoiceArg::Choice0 => 0,
            ChoiceArg::Choice1 => 1,
        })
    }

    fn dbus_arg_try_into(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ChoiceArg::Choice0),
            1 => Ok(ChoiceArg::Choice1),
            _ => Err("Cannot properly handle error for enum conversion"),
        }
    }
}

#[test]
fn conversion() -> Result<(), Box<dyn Error>> {
    let args_named = ArgsNamed::default();
    let args_unnamed = ArgsUnnamed::default();

    let args_named_tuple: ArgsNamedTuple = args_named.try_into()?;
    let args_unnamed_tuple: ArgsUnnamedTuple = args_unnamed.try_into()?;

    let _args_named_converted: ArgsNamed = args_named_tuple.try_into()?;
    let _args_unnamed_converted: ArgsUnnamed = args_unnamed_tuple.try_into()?;

    Ok(())
}
