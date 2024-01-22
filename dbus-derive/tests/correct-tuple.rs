use dbus_derive::DbusArgs;
use dbus_traits::DbusArg;

#[derive(DbusArgs)]
pub struct ArgsNamed<'a> {
    pub x: i32,
    pub y: u32,
    pub foo: String,
    pub bar: &'a str,
    pub arr: &'a [bool],
    pub bytes: [u8; 4],
    pub vec: Vec<f64>,
    pub nested_arg: NestedArg,
    pub nested_args: Vec<NestedArg>,
    #[dbus_args(mapped_type = u8)]
    pub choice_arg: ChoiceArg,
}

#[derive(DbusArgs)]
pub struct ArgsUnnamed<'a>(
    pub i32,
    pub u32,
    pub String,
    pub &'a str,
    pub &'a [bool],
    pub [u8; 4],
    pub Vec<f64>,
    pub NestedArg,
    pub Vec<NestedArg>,
    #[dbus_args(mapped_type = u8)] pub ChoiceArg,
);

#[derive(DbusArgs)]
pub struct NestedArg {
    pub arg1: i32,
    pub arg2: i32,
}

pub enum ChoiceArg {
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
fn conversion() {
    #[allow(clippy::type_complexity)]
    let args: (
        i32,
        u32,
        String,
        &str,
        &[bool],
        [u8; 4],
        Vec<f64>,
        (i32, i32),
        Vec<(i32, i32)>,
        u8,
    ) = (
        0,
        0u32,
        "foo".to_string(),
        "bar",
        &[true, false],
        [0xDE, 0xAD, 0xBE, 0xEF],
        vec![0.0f64],
        (0, 0),
        vec![(1, 1), (2, 2)],
        0,
    );

    let args_named: ArgsNamed = args.clone().try_into().unwrap();
    let args_unnamed: ArgsUnnamed = args.clone().try_into().unwrap();

    let args_named_tuple: ArgsNamedTuple = args_named.try_into().unwrap();
    let _: ArgsUnnamedTuple = args_unnamed.try_into().unwrap();

    debug_assert_eq!(args, args_named_tuple);
}
