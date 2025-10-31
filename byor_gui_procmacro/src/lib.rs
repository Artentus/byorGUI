use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    Data, DeriveInput, Field, Fields, GenericArgument, Ident, PathArguments, Type,
    parse_macro_input,
};

fn expand_field(field: &Field) -> TokenStream2 {
    let field_name = field.ident.as_ref().unwrap();

    let initial_function_name =
        Ident::new(&format!("with_initial_{field_name}"), field_name.span());
    let inherit_function_name = Ident::new(&format!("inherit_{field_name}"), field_name.span());
    let with_function_name = Ident::new(&format!("with_{field_name}"), field_name.span());

    if let Type::Path(type_path) = &field.ty
        && type_path.qself.is_none()
        && (type_path.path.segments.len() == 1)
    {
        let field_type = &type_path.path.segments[0];

        if let PathArguments::AngleBracketed(generic_arguments) = &field_type.arguments {
            let generic_type_count = generic_arguments
                .args
                .iter()
                .filter(|arg| matches!(arg, GenericArgument::Type(_)))
                .count();

            if generic_type_count == 1 {
                let inner_type = &generic_arguments
                    .args
                    .iter()
                    .filter_map(|arg| match arg {
                        GenericArgument::Type(inner_type) => Some(inner_type),
                        _ => None,
                    })
                    .next()
                    .unwrap();

                return quote_spanned! {
                    field.span() =>
                    #[must_use]
                    #[inline]
                    pub fn #initial_function_name(self) -> Self {
                        Self {
                            #field_name: Property::Initial,
                            ..self
                        }
                    }

                    #[must_use]
                    #[inline]
                    pub fn #inherit_function_name(self) -> Self {
                        Self {
                            #field_name: Property::Inherit,
                            ..self
                        }
                    }

                    #[must_use]
                    #[inline]
                    pub fn #with_function_name(self, #field_name: impl Into<#inner_type>) -> Self {
                        Self {
                            #field_name: Property::Value(#field_name.into()),
                            ..self
                        }
                    }
                };
            }
        }
    }

    quote_spanned! {
        field.ty.span() =>
        compile_error!("invalid field type");
    }
}

#[proc_macro_derive(StyleBuilder)]
pub fn style_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(_) => {
                let struct_name = &input.ident;
                let builder_functions: Vec<_> =
                    data_struct.fields.iter().map(expand_field).collect();

                quote! {
                    impl #struct_name {
                        #(#builder_functions)*
                    }
                }
            }
            _ => {
                quote_spanned! {
                    data_struct.fields.span() =>
                    compile_error!("expected named fields");
                }
            }
        },
        Data::Enum(data_enum) => {
            quote_spanned! {
                data_enum.enum_token.span() =>
                compile_error!("expected struct");
            }
        }
        Data::Union(data_union) => {
            quote_spanned! {
                data_union.union_token.span() =>
                compile_error!("expected struct");
            }
        }
    };

    TokenStream::from(expanded)
}
