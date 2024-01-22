use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Attribute, Data, DataStruct,
    DeriveInput, Expr, ExprAssign, Fields, GenericArgument, Index, PathSegment, Token, Type,
    TypeGenerics, TypePath,
};

#[proc_macro_derive(DbusArgs, attributes(dbus_args))]
#[proc_macro_error]
pub fn derive_dbus_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let input_struct = derive_input_struct(&input);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let tuple_name = format_ident!("{name}Tuple");
    let tuple_types = struct_to_tuple_types(input_struct);
    let try_from_tuple = try_from_tuple_method(&tuple_name, input_struct, &ty_generics);
    let try_from_struct = try_from_struct_method(name, input_struct, &ty_generics);

    let tokens = quote! {
        type #tuple_name #ty_generics = (#tuple_types);

        impl #impl_generics ::core::convert::TryFrom<#tuple_name #ty_generics> for #name #ty_generics #where_clause {
            type Error = ::std::boxed::Box<dyn ::std::error::Error>;

            #try_from_tuple
        }

        impl #impl_generics ::core::convert::TryFrom<#name #ty_generics> for #tuple_name #ty_generics #where_clause {
            type Error = ::std::boxed::Box<dyn ::std::error::Error>;

            #try_from_struct
        }

        impl #impl_generics ::dbus_traits::DbusArg<#tuple_name #ty_generics> for #name #ty_generics #where_clause {
            type Error = ::std::boxed::Box<dyn ::std::error::Error>;

            fn dbus_arg_try_from(self) -> ::core::result::Result<#tuple_name #ty_generics, Self::Error> {
                self.try_into()
            }
            fn dbus_arg_try_into(value: #tuple_name #ty_generics) -> ::core::result::Result<Self, Self::Error> {
                value.try_into()
            }
        }
    };
    tokens.into()
}

/// Extracts struct info from derive input
fn derive_input_struct(input: &DeriveInput) -> &DataStruct {
    match &input.data {
        Data::Struct(data) => data,
        Data::Enum(enum_data) => {
            abort! {
                enum_data.enum_token.span,
                "dbus_derive::DbusArgs does not support enum"
            }
        }
        Data::Union(union_data) => {
            abort! {
                union_data.union_token.span,
                "dbus_derive::DbusArgs does not support union"
            }
        }
    }
}

/// Takes field types from struct and joins them into comma separated tokens
fn struct_to_tuple_types(data_struct: &DataStruct) -> TokenStream {
    let fields = match data_struct.fields {
        Fields::Named(ref fields) => &fields.named,
        Fields::Unnamed(ref fields) => &fields.unnamed,
        Fields::Unit => {
            abort! {
                data_struct.fields.span(),
                "dbus_derive::DbusArgs does not support unit structs"
            }
        }
    };
    let field_types = fields.iter().map(|field| {
        if let Some(dbus_args_params) = parse_field_attributes(&field.attrs) {
            if let Some(mapped_type) = dbus_args_params.mapped_type {
                return mapped_type;
            };
        };
        field_to_tuple_type(&field.ty)
    });
    quote!(#(#field_types),*)
}

fn parse_field_attributes(attrs: &[Attribute]) -> Option<DbusArgsAttributeParameters> {
    for attr in attrs {
        match &attr.meta {
            syn::Meta::List(ref list) => {
                if list
                    .path
                    .segments
                    .last()
                    .is_some_and(|seg| seg.ident == "dbus_args")
                {
                    return Some(parse_dbus_args_attribute(attr));
                }
            }
            _ => continue,
        }
    }
    None
}

#[derive(Default)]
struct DbusArgsAttributeParameters {
    mapped_type: Option<TokenStream>,
}

fn parse_dbus_args_attribute(attr: &Attribute) -> DbusArgsAttributeParameters {
    let mut params = DbusArgsAttributeParameters::default();
    let Ok(args) = attr.parse_args_with(Punctuated::<ExprAssign, Token![,]>::parse_terminated)
    else {
        abort! {
            attr.span(),
            "expected dbus_args(key1 = val1, key2 = val2)"
        }
    };
    for arg in args {
        let Expr::Path(ref path) = *arg.left else {
            abort! {
                arg.left.span(),
                "expected a valid identifier"
            }
        };
        if path.path.segments.len() != 1 {
            abort! {
                arg.left.span(),
                "expected a valid identifier"
            }
        }
        let key = path.path.segments.first().unwrap();
        if key.ident == "mapped_type" {
            if params.mapped_type.is_some() {
                abort! {
                    arg.left.span(),
                    "this key should be unique"
                }
            }
            let Expr::Path(ref mapped_type) = *arg.right else {
                abort! {
                    arg.right.span(),
                    "this should be a path"
                }
            };
            params.mapped_type = Some(quote!(#mapped_type));
        } else {
            abort! {
                arg.left.span(),
                "unrecognized parameter"
            }
        }
    }
    params
}

/// Types that have dbus::arg::Arg implemented for them directly, or for reference to them.
/// Used to differentiate from user defined types, in which case "Tuple" is appended to type name
const DBUS_NATIVE_TYPES: [&str; 13] = [
    "u8", "u16", "u32", "u64", "i16", "i32", "i64", "f64", "bool", "String", "str", "CStr", "File",
];
/// Generic types that have dbus::arg::Arg implemented for them.
/// Used to differentiate from user defined types, in which case "Tuple" is appended to type name
const DBUS_CONTAINER_TYPES: [&str; 3] = ["Vec", "Box", "Arc"];

fn field_to_tuple_type(field_type: &Type) -> TokenStream {
    match &field_type {
        Type::Path(field_path) => {
            let Some(ty_segment) = field_path.path.segments.last() else {
                abort! {
                    field_path.span(),
                    "dbus_derive::DbusArgs expects TypePath to have at least 1 segment"
                }
            };

            if DBUS_NATIVE_TYPES.iter().any(|ty| ty_segment.ident == ty) {
                // Type directly implements dbus::arg::Arg - leave as it is
                quote_spanned!(field_path.span() => #field_path)
            } else if DBUS_CONTAINER_TYPES.iter().any(|ty| ty_segment.ident == ty) {
                // Type is a container that implements dbus::arg::Arg - look into generic argument
                // while leaving container as it is
                field_generic_to_tuple_type(field_path, ty_segment)
            } else {
                // This type is not known to implement dbus::arg::Arg - assume this is another
                // struct that has derive(DbusArgs)
                let tuple_name = format_ident!("{}Tuple", ty_segment.ident);
                let mut modified_field = field_path.clone();
                if let Some(seg) = modified_field.path.segments.last_mut() {
                    seg.ident = tuple_name;
                }
                quote_spanned!(field_path.span() => #modified_field)
            }
        }
        Type::Reference(field_ref) => {
            let mut modified_field = field_ref.clone();
            modified_field.elem = Box::new(Type::Verbatim(field_to_tuple_type(&field_ref.elem)));
            quote_spanned!(field_type.span() => #modified_field)
        }
        Type::Slice(field_slice) => {
            let mut modified_field = field_slice.clone();
            modified_field.elem = Box::new(Type::Verbatim(field_to_tuple_type(&field_slice.elem)));
            quote_spanned!(field_type.span() => #modified_field)
        }
        Type::Array(field_array) => {
            let mut modified_field = field_array.clone();
            modified_field.elem = Box::new(Type::Verbatim(field_to_tuple_type(&field_array.elem)));
            quote_spanned!(field_type.span() => #modified_field)
        }
        _ => {
            abort! {
                field_type.span(),
                "dbus_derive::DbusArgs does not support this field type"
            }
        }
    }
}

fn field_generic_to_tuple_type(field_path: &TypePath, segment: &PathSegment) -> TokenStream {
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(generic_args) => {
            let Some(generic_arg) = generic_args.args.first() else {
                abort! {
                    field_path.span(),
                    "dbus_derive::DbusArgs expects this type to have at least 1 generic argument"
                }
            };
            let GenericArgument::Type(nested_ty) = generic_arg else {
                abort! {
                    generic_arg.span(),
                    "dbus_derive::DbusArgs expects this to be a type"
                }
            };
            let nested_tuple = field_to_tuple_type(nested_ty);
            let mut modified_args = generic_args.clone();
            if let Some(arg) = modified_args.args.first_mut() {
                *arg = GenericArgument::Type(Type::Verbatim(nested_tuple))
            };
            let mut modified_field = field_path.clone();
            if let Some(seg) = modified_field.path.segments.last_mut() {
                seg.arguments = syn::PathArguments::AngleBracketed(modified_args);
            }
            quote_spanned!(field_path.span() => #modified_field)
        }
        syn::PathArguments::Parenthesized(_) => {
            abort! {
                field_path.span(),
                "dbus_derive::DbusArgs does not support types with parenthesized arguments"
            }
        }
        syn::PathArguments::None => {
            abort! {
                field_path.span(),
                "dbus_derive::DbusArgs expects this type to have a generic argument"
            }
        }
    }
}

/// Generate a fn try_from(val: Tuple) -> Result<StructName, _> for a Dbus Args struct
fn try_from_tuple_method(
    tuple_name: &Ident,
    data_struct: &DataStruct,
    ty_generics: &TypeGenerics,
) -> TokenStream {
    match data_struct.fields {
        Fields::Named(ref fields) => {
            let field_assignments = fields.named.iter().enumerate().map(|(i, field)| {
                let i = Index::from(i);
                let field_name = field
                    .ident
                    .clone()
                    .expect("Got a field without a name in Fields::Named");
                quote_spanned! { field.span() =>
                    #field_name: ::dbus_traits::DbusArg::dbus_arg_try_into(value.#i)?
                }
            });
            quote! {
                fn try_from(value: #tuple_name #ty_generics) -> ::core::result::Result<Self, Self::Error> {
                    Ok(Self { #(#field_assignments),* })
                }
            }
        }
        Fields::Unnamed(ref fields) => {
            let field_assignments = fields.unnamed.iter().enumerate().map(|(i, field)| {
                let i = Index::from(i);
                quote_spanned! { field.span() =>
                    ::dbus_traits::DbusArg::dbus_arg_try_into(value.#i)?
                }
            });
            quote! {
                fn try_from(value: #tuple_name #ty_generics) -> ::core::result::Result<Self, Self::Error> {
                    Ok(Self(#(#field_assignments),*))
                }
            }
        }
        Fields::Unit => {
            abort! {
                data_struct.fields.span(),
                "dbus_derive::DbusArgs does not support unit structs"
            }
        }
    }
}

/// Generate a fn try_from(val: StructName) -> Resule<Tuple, _> for a Dbus Args struct
fn try_from_struct_method(
    struct_name: &Ident,
    data_struct: &DataStruct,
    ty_generics: &TypeGenerics,
) -> TokenStream {
    let fields = match data_struct.fields {
        Fields::Named(ref fields) => {
            let field_assignments = fields.named.iter().map(|field| {
                let field_name = field
                    .ident
                    .clone()
                    .expect("Got a field without a name in Fields::Named");
                quote_spanned! { field.span() =>
                    ::dbus_traits::DbusArg::dbus_arg_try_from(value.#field_name)?
                }
            });
            quote!(#(#field_assignments),*)
        }
        Fields::Unnamed(ref fields) => {
            let field_assignments = fields.unnamed.iter().enumerate().map(|(i, field)| {
                let i = Index::from(i);
                quote_spanned! { field.span() =>
                    ::dbus_traits::DbusArg::dbus_arg_try_from(value.#i)?
                }
            });
            quote!(#(#field_assignments),*)
        }
        Fields::Unit => {
            abort! {
                data_struct.fields.span(),
                "dbus_derive::DbusArgs does not support unit structs"
            }
        }
    };
    quote! {
        fn try_from(value: #struct_name #ty_generics) -> ::core::result::Result<Self, Self::Error> {
            Ok((#fields))
        }
    }
}
