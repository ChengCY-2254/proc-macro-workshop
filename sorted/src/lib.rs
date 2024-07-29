use proc_macro::TokenStream;
use quote::ToTokens;
use syn::ItemEnum;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;
    let ast = syn::parse_macro_input!(input as syn::Item);
    eprintln!("{:#?}", ast);
    match get_enum(&ast) {
        Ok(e) => handle_enum(e),
        Err(e) => return e.into_compile_error().into(),
    }
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}
/// 对获取的枚举进行处理
fn handle_enum(e: &ItemEnum) -> syn::Result<proc_macro2::TokenStream> {
    Ok(e.to_token_stream())
}

/// 检查是否为枚举类型
fn get_enum(expr: &syn::Item) -> syn::Result<&syn::ItemEnum> {
    if let syn::Item::Enum(e) = expr {
        return Ok(e);
    }
    Err(syn::Error::new_spanned(
        expr,
        "#[sorted] cannot be applied to things other than enum",
    ))
}
