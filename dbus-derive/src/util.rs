use darling::ast::Style;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Type};

/// Returns struct constructor that is appropriate for given struct style
pub fn fields_to_constructor(span: &Span, style: &Style, var_names: &[Ident]) -> TokenStream {
    match style {
        Style::Struct => {
            quote! {
                Self { #(#var_names),* }
            }
        }
        Style::Tuple => {
            quote! {
                Self ( #(#var_names),* )
            }
        }
        Style::Unit => {
            abort!(span, "Unit structs not supported")
        }
    }
}

/// Returns array of identifiers that could be used as variable name for each field
pub fn fields_to_var_idents(
    span: &Span,
    style: &Style,
    field_idents: &[Option<Ident>],
) -> Vec<Ident> {
    field_idents
        .iter()
        .enumerate()
        .map(|(idx, field)| match style {
            Style::Struct => field.clone().expect("Fields in structs should have names"),
            Style::Tuple => format_ident!("arg{idx}"),
            Style::Unit => abort!(span, "Unit structs not supported"),
        })
        .collect()
}

/// Extracts a generic argument idx from ty and parses it as syn::Type
pub fn ty_generic_to_ty_contained(ty: &Type, idx: usize) -> Type {
    match ty {
        Type::Path(ty_path) => {
            let segments = &ty_path.path.segments;
            if segments.len() < idx + 1 {
                abort!(ty, "Path should not be empty");
            }
            let segment = segments.last().unwrap(/* Verified above */);
            let args = match &segment.arguments {
                syn::PathArguments::AngleBracketed(args) => args,
                _ => abort!(ty, "No generic arguments, expected at least {}", idx + 1),
            };
            let Some(arg_contained) = args.args.iter().nth(idx) else {
                abort!(
                    ty,
                    "Not enough generic arguments, expected at least {}",
                    idx + 1
                );
            };
            let Ok(ty_contained) = syn::parse::<Type>(arg_contained.to_token_stream().into())
            else {
                abort!(
                    ty,
                    "Argument {} is not a valid type",
                    arg_contained.to_token_stream().to_string()
                );
            };
            ty_contained
        }
        _ => abort!(ty, "Expected to be a generic type"),
    }
}
