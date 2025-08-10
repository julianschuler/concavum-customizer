//! The `show_derive` crate defines a proc macro for deriving the `Show` trait.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Fields,
    FieldsNamed, Lit, Meta, MetaNameValue,
};

/// Derives the `Show` trait for a struct.
///
/// # Panics
///
/// Panics if applied to structs without named fields or if fields are undocumented.
#[proc_macro_derive(Show)]
pub fn derive_show(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = data
    else {
        panic!("`Show` can only be derived for structs with named fields")
    };

    let section_title = title_from_struct_name(&ident.to_string());
    let parameters = named.iter().map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("named field should have an identifier");

        let docstring = field
            .attrs
            .iter()
            .find_map(|attribute| {
                if let Attribute {
                    meta:
                        Meta::NameValue(MetaNameValue {
                            value:
                                Expr::Lit(ExprLit {
                                    lit: Lit::Str(string),
                                    ..
                                }),
                            ..
                        }),
                    ..
                } = attribute
                {
                    Some(string.value())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                format!("field `{ident}` is missing a description in form of a docstring")
            });

        let description = docstring.trim_start().trim_end_matches('.');
        let field_name = name_from_struct_field(&ident.to_string());

        quote! {
            changed |= self.#ident.show_with_name_and_description(ui, #field_name, #description);
        }
    });

    quote! {
        impl Show for #ident {
            const TITLE: &'static str = #section_title;

            fn show(&mut self, ui: &mut show::egui::Ui) -> bool {
                let mut changed = false;

                #(#parameters)*

                changed
            }
        }
    }
    .into()
}

/// Creates the title of a section from a struct name.
fn title_from_struct_name(string: &str) -> String {
    string
        .chars()
        .enumerate()
        .flat_map(|(index, char)| {
            if char.is_ascii_uppercase() {
                if index == 0 {
                    [Some(char), None]
                } else {
                    [Some(' '), Some(char.to_ascii_lowercase())]
                }
            } else {
                [Some(char), None]
            }
        })
        .flatten()
        .collect()
}

/// Creates the name of a parameter from a struct field name.
fn name_from_struct_field(string: &str) -> String {
    string
        .replace("pcb", "PCB")
        .replace("ffc", "FFC")
        .chars()
        .enumerate()
        .map(|(index, char)| {
            if index == 0 {
                char.to_ascii_uppercase()
            } else if char == '_' {
                ' '
            } else {
                char
            }
        })
        .collect()
}
