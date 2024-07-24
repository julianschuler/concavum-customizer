//! The `show_derive` crate defines a proc macro for deriving the `Show` trait.

extern crate proc_macro;

use convert_case::{Case, Casing};
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

    let section_name = convert_case(&ident);

    let sections = named.iter().map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("named field should have an identifer");

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
        let field_name = convert_case(&ident);

        quote! {
            self.#ident.show_with_name_and_description(ui, #field_name, #description);
        }
    });

    quote! {
        impl Show for #ident {
            fn show(&mut self, ui: &mut show::egui::Ui) {
                show::parameters_section(ui, #section_name, |ui| {
                    #(#sections)*
                });
            }
        }
    }
    .into()
}

fn convert_case(value: &impl ToString) -> String {
    value.to_string().to_case(Case::Title)
}
