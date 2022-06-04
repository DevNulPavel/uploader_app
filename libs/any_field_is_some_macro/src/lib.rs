// Включаем фичу трассировки макросов
// #![feature(trace_macros)]

// Непосредственно само включение трассировки макросов
// trace_macros!(true);

use proc_macro2::Ident;
// use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, ItemStruct, Type};

#[proc_macro_attribute]
pub fn any_field_is_some(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Парсим сначала в токены структуры
    let struct_data = parse_macro_input!(input as ItemStruct);

    // Проверяем, что мы работаем со структурой
    for field in struct_data.fields.iter() {
        if let (Some(name), Type::Path(type_path)) = (&field.ident, &field.ty) {
            // let mut punct: Punctuated<PathSegment, Token!(::)> = Punctuated::new();
            // punct.push_value(PathSegment {
            //     arguments: Default::default(),
            //     ident: Ident::new("std", Span::call_site()),
            // });
            // punct.push_value(PathSegment {
            //     arguments: Default::default(),
            //     ident: Ident::new("option", Span::call_site()),
            // });
            // punct.push_value(PathSegment {
            //     arguments: Default::default(),
            //     ident: Ident::new("Option", Span::call_site()),
            // });

            // let punct = parse_quote!(std::option::Option<_>);
            // if !type_path.path.segments.eq(&punct) {
            //     panic!(
            //         r#"Field with name "{}" must be "{}", found "{}""#,
            //         name, punct.to_token_stream(), type_path.path.segments.to_token_stream()
            //     );
            // }

            if let Some(last) = type_path.path.segments.last() {
                // let option_type: Ident = Ident::new("Option", Span::call_site());
                let option_type: Ident = parse_quote!(Option);
                if !last.ident.eq(&option_type) {
                    panic!(
                        r#"Field with name "{}" must be "{}", found "{}""#,
                        name, option_type, last.ident
                    );
                }
            } else {
                panic!("Type is missing for field: {}", name);
            }
        }
    }

    // Возвращаем выходные токены компилятору
    proc_macro::TokenStream::from(quote::quote!(#struct_data))
}
