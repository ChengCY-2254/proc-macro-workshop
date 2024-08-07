use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;
/// 为枚举类型生成枚举值的常量
/// ```rust
/// #[derive(derive_enums::Enums)]
/// pub enum Html {
///     Body,
///     Div,
///     H1,
/// }
/// ```
/// 将会产生一个values的方法，返回一个包含枚举值的字符串数组
/// ```rust
///#  pub enum Html {
///#      Body,
///#      Div,
///#      H1,
///#  }
///  const HTML_VALUES: &[&str] = &["Body", "Div", "H1"];
///  impl Html {
///      pub fn values_str() -> &'static [&'static str] {
///          HTML_VALUES
///      }
///  }
/// ```
#[proc_macro_derive(Enums)]
pub fn enums(input: TokenStream) -> TokenStream {
    let mut ret = TokenStream2::new();
    let enum_input = parse_macro_input!(input as syn::DeriveInput);
    eprintln!("{:#?}", enum_input);
    ret.extend(enum_visitor(&enum_input));
    ret.into()
}

fn enum_visitor(enum_input: &syn::DeriveInput) -> TokenStream2 {
    let vis = &enum_input.vis;
    let raw_enum_ident = &enum_input.ident;
    let enum_ident = append_str_ident(
        raw_enum_ident.to_string().to_uppercase().as_str(),
        "_VALUES",
        raw_enum_ident.span(),
    );

    if let syn::Data::Enum(syn::DataEnum { variants, .. }) = &enum_input.data {
        let enum_values = variants.into_iter().map(|field| {
            let field_type = &field.ident.to_string();
            quote! {
                #field_type
            }
        });
        //生成一个const 常量并实现一个函数来返回这个常量
        quote! {
            const #enum_ident:&[&str] = &[#(#enum_values),*];
            impl #raw_enum_ident{
                #vis fn values()->&'static [&'static str]{
                    #enum_ident
                }
            }
        }
    } else {
        panic!("Not an enum")
    }
}

fn append_str_ident(ident: &str, s: &str, span: proc_macro2::Span) -> syn::Ident {
    syn::Ident::new(format!("{}{}", ident, s).as_str(), span)
}
