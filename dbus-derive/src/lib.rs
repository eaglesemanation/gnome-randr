mod derive_args;
mod derive_enum;
mod derive_propmap;
mod derive_struct;
mod util;

use darling::FromDeriveInput;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

use crate::derive_args::{derive_args, DbusArgs};
use crate::derive_enum::{derive_enum, DbusEnum};
use crate::derive_propmap::{derive_propmap, DbusPropmap};
use crate::derive_struct::{derive_struct, DbusStruct};
use crate::util::derive_input_style_span;

#[proc_macro_derive(DbusStruct, attributes(dbus_struct))]
#[proc_macro_error]
pub fn derive_dbus_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input = match DbusStruct::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err
                .with_span(&derive_input_style_span(input))
                .write_errors()
                .into();
        }
    };
    derive_struct(input).into()
}

#[proc_macro_derive(DbusArgs, attributes(dbus_args))]
#[proc_macro_error]
pub fn derive_dbus_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input = match DbusArgs::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err
                .with_span(&derive_input_style_span(input))
                .write_errors()
                .into();
        }
    };
    derive_args(input).into()
}

#[proc_macro_derive(DbusEnum, attributes(dbus_enum))]
#[proc_macro_error]
pub fn derive_dbus_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input = match DbusEnum::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err
                .with_span(&derive_input_style_span(input))
                .write_errors()
                .into();
        }
    };
    derive_enum(input).into()
}

#[proc_macro_derive(DbusPropMap, attributes(dbus_propmap))]
#[proc_macro_error]
pub fn derive_dbus_propmap(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input = match DbusPropmap::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err
                .with_span(&derive_input_style_span(input))
                .write_errors()
                .into();
        }
    };
    derive_propmap(input).into()
}
