use darling::{ast, util::SpannedValue, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, DeriveInput, GenericArgument, Index, PathSegment, Type,
    TypePath,
};

#[derive(Debug, FromField)]
#[darling(attributes(dbus_arg))]
struct DbusArgsField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    derived: bool,
    target_type: Option<Type>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(dbus_arg),
    supports(struct_named, struct_tuple, struct_newtype)
)]
struct DbusArgs {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<darling::util::Ignored, SpannedValue<DbusArgsField>>,
}

#[proc_macro_derive(DbusArgs, attributes(dbus_arg))]
#[proc_macro_error]
pub fn derive_dbus_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DbusArgs {
        ref ident,
        ref generics,
        data,
    } = match DbusArgs::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };
    let data = data.take_struct().unwrap(/* using #[darling(supports(struct_named, struct_tuple, struct_newtype))], should fail on previous step if enum */);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let tuple_name = format_ident!("{ident}Tuple");
    let tuple_name = quote!(#tuple_name #ty_generics);
    let input_name = quote!(#ident #ty_generics);

    let tuple_types = fields_to_tuple_types(&data);
    let try_from_tuple = try_from_tuple_method(&tuple_name, &data);
    let try_from_struct = try_from_struct_method(&input_name, &data);

    let tokens = quote! {
        type #tuple_name = (#tuple_types);

        #[automatically_derived]
        impl #impl_generics ::core::convert::TryFrom<#tuple_name> for #input_name #where_clause {
            type Error = ::std::boxed::Box<dyn ::std::error::Error>;

            #try_from_tuple
        }

        #[automatically_derived]
        impl #impl_generics ::core::convert::TryFrom<#input_name> for #tuple_name #where_clause {
            type Error = ::std::boxed::Box<dyn ::std::error::Error>;

            #try_from_struct
        }

        #[automatically_derived]
        impl #impl_generics ::dbus_traits::DbusArg<#tuple_name> for #input_name #where_clause {
            type Error = ::std::boxed::Box<dyn ::std::error::Error>;

            fn dbus_arg_try_from(self) -> ::core::result::Result<#tuple_name, Self::Error> {
                self.try_into()
            }
            fn dbus_arg_try_into(value: #tuple_name) -> ::core::result::Result<Self, Self::Error> {
                value.try_into()
            }
        }
    };
    tokens.into()
}

/// Takes field types from struct and joins them into comma separated tokens
fn fields_to_tuple_types(fields: &ast::Fields<SpannedValue<DbusArgsField>>) -> TokenStream {
    let field_types = fields.iter().map(|field| {
        if let Some(ty) = &field.target_type {
            field_to_tuple_type(ty, false)
        } else {
            field_to_tuple_type(&field.ty, field.derived)
        }
    });
    quote!(#(#field_types),*)
}

/// Generic types that have dbus::arg::Arg implemented for them.
const DBUS_CONTAINER_TYPES: [(&str, usize); 4] =
    [("Vec", 0), ("Box", 0), ("Arc", 0), ("HashMap", 1)];

fn field_to_tuple_type(field_type: &Type, field_derived: bool) -> TokenStream {
    match &field_type {
        Type::Path(field_path) => {
            let Some(ty_segment) = field_path.path.segments.last() else {
                abort! {
                    field_path,
                    "dbus_derive::DbusArgs expects TypePath to have at least 1 segment"
                }
            };

            if let Some((_, generic_idx)) = DBUS_CONTAINER_TYPES
                .iter()
                .find(|(ty, _)| ty_segment.ident == ty)
            {
                field_generic_to_tuple_type(field_path, ty_segment, *generic_idx, field_derived)
            } else if field_derived {
                let tuple_name = format_ident!("{}Tuple", ty_segment.ident);
                let mut modified_field = field_path.clone();
                if let Some(seg) = modified_field.path.segments.last_mut() {
                    seg.ident = tuple_name;
                }
                quote_spanned!(field_path.span() => #modified_field)
            } else {
                quote_spanned!(field_path.span() => #field_path)
            }
        }
        Type::Reference(field_ref) => {
            let mut modified_field = field_ref.clone();
            modified_field.elem = Box::new(Type::Verbatim(field_to_tuple_type(
                &field_ref.elem,
                field_derived,
            )));
            quote_spanned!(field_type.span() => #modified_field)
        }
        Type::Slice(field_slice) => {
            let mut modified_field = field_slice.clone();
            modified_field.elem = Box::new(Type::Verbatim(field_to_tuple_type(
                &field_slice.elem,
                field_derived,
            )));
            quote_spanned!(field_type.span() => #modified_field)
        }
        Type::Array(field_array) => {
            let mut modified_field = field_array.clone();
            modified_field.elem = Box::new(Type::Verbatim(field_to_tuple_type(
                &field_array.elem,
                field_derived,
            )));
            quote_spanned!(field_type.span() => #modified_field)
        }
        _ => {
            abort! {
                field_type,
                "dbus_derive::DbusArgs does not support this field type"
            }
        }
    }
}

fn field_generic_to_tuple_type(
    field_path: &TypePath,
    segment: &PathSegment,
    generic_idx: usize,
    field_derived: bool,
) -> TokenStream {
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(generic_args) => {
            let Some(generic_arg) = generic_args.args.iter().nth(generic_idx) else {
                abort! {
                    field_path,
                    "dbus_derive::DbusArgs expects this type to have at least {} generic argument(s)",
                    generic_idx + 1
                }
            };
            let GenericArgument::Type(nested_ty) = generic_arg else {
                abort! {
                    generic_arg,
                    "dbus_derive::DbusArgs expects this to be a type"
                }
            };
            let nested_tuple = field_to_tuple_type(nested_ty, field_derived);
            let mut modified_args = generic_args.clone();
            if let Some(arg) = modified_args.args.iter_mut().nth(generic_idx) {
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
                field_path,
                "dbus_derive::DbusArgs does not support types with parenthesized arguments"
            }
        }
        syn::PathArguments::None => {
            abort! {
                field_path,
                "dbus_derive::DbusArgs expects this type to have a generic argument"
            }
        }
    }
}

/// Generate a fn try_from(val: Tuple) -> Result<StructName, _> for a Dbus Args struct
fn try_from_tuple_method(
    tuple_name: &TokenStream,
    fields: &ast::Fields<SpannedValue<DbusArgsField>>,
) -> TokenStream {
    match fields.style {
        ast::Style::Struct => {
            let field_assignments = fields.iter().enumerate().map(|(i, field)| {
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
                fn try_from(value: #tuple_name) -> ::core::result::Result<Self, Self::Error> {
                    Ok(Self { #(#field_assignments),* })
                }
            }
        }
        ast::Style::Tuple => {
            let field_assignments = fields.iter().enumerate().map(|(i, field)| {
                let i = Index::from(i);
                quote_spanned! { field.span() =>
                    ::dbus_traits::DbusArg::dbus_arg_try_into(value.#i)?
                }
            });
            quote! {
                fn try_from(value: #tuple_name) -> ::core::result::Result<Self, Self::Error> {
                    Ok(Self(#(#field_assignments),*))
                }
            }
        }
        ast::Style::Unit => {
            unreachable!(
                "#[darling(supports(struct_named, struct_tuple, struct_newtype))] in dbus-derive::DbusArgs should've filtered fields of Style::Unit"
            )
        }
    }
}

/// Generate a fn try_from(val: StructName) -> Resule<Tuple, _> for a Dbus Args struct
fn try_from_struct_method(
    struct_name: &TokenStream,
    fields: &ast::Fields<SpannedValue<DbusArgsField>>,
) -> TokenStream {
    let fields = match fields.style {
        ast::Style::Struct => {
            let field_assignments = fields.iter().map(|field| {
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
        ast::Style::Tuple => {
            let field_assignments = fields.iter().enumerate().map(|(i, field)| {
                let i = Index::from(i);
                quote_spanned! { field.span() =>
                    ::dbus_traits::DbusArg::dbus_arg_try_from(value.#i)?
                }
            });
            quote!(#(#field_assignments),*)
        }
        ast::Style::Unit => {
            unreachable!(
                "#[darling(supports(struct_named, struct_tuple, struct_newtype))] in dbus-derive::DbusArgs should've filtered fields of Style::Unit"
            )
        }
    };
    quote! {
        fn try_from(value: #struct_name) -> ::core::result::Result<Self, Self::Error> {
            Ok((#fields))
        }
    }
}
