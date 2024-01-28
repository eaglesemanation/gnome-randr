mod derive_args;
mod derive_enum;
mod derive_propmap;
mod util;

use darling::FromDeriveInput;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

use crate::derive_args::{derive_args, DbusArgs};
use crate::derive_enum::{derive_enum, DbusEnum};
use crate::derive_propmap::{derive_propmap, DbusPropmap};

#[proc_macro_derive(DbusArgs, attributes(dbus_arg))]
#[proc_macro_error]
pub fn derive_dbus_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = match DbusArgs::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };
    derive_args(input).into()
}

#[proc_macro_derive(DbusEnum, attributes(dbus_enum))]
#[proc_macro_error]
pub fn derive_dbus_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = match DbusEnum::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };
    derive_enum(input).into()
}

#[proc_macro_derive(DbusPropMap, attributes(dbus_propmap))]
#[proc_macro_error]
pub fn derive_dbus_propmap(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = match DbusPropmap::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };
    derive_propmap(input).into()
}
