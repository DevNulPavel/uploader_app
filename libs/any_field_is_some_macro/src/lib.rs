use proc_macro2::{Ident, Span};
use syn::{parse_macro_input, ItemStruct, Type};

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
            if let Some(last) = type_path.path.segments.last() {
                let option_type = Ident::new("Option", Span::call_site());
                if !last.ident.eq(&option_type) {
                    panic!(
                        r#"Field with name "{}" must be "{}", found "{}""#,
                        name, option_type, last.ident
                    );
                }
            }
        }
    }

    // Возвращаем выходные токены компилятору
    proc_macro::TokenStream::from(quote::quote!(#struct_data))
}
