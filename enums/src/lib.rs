use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error as SynError, parse_macro_input};

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
    // eprintln!("{:#?}", enum_input);
    match EnumVisitorConfig::try_from(&enum_input) {
        Ok(config) => ret.extend(generator_enum_visitor(config)),
        Err(e) => ret.extend(e.into_compile_error()),
    };
    ret.into()
}

struct EnumVisitorConfig<'a> {
    ///枚举的可访问性
    pub vis: &'a syn::Visibility,
    ///枚举名称
    pub raw_enum_ident: &'a syn::Ident,
    ///枚举类型的ast类型
    pub enum_variants: &'a syn::DataEnum,
}
impl<'a> EnumVisitorConfig<'a> {
    ///为枚举名称添加_VALUES后缀以用作常量类型使用
    fn const_values_ident(&self) -> syn::Ident {
        utils::append_str_ident(
            self.raw_enum_ident.to_string().to_uppercase().as_str(),
            "_VALUES",
            self.raw_enum_ident.span(),
        )
    }
}
impl<'a> TryFrom<&'a syn::DeriveInput> for EnumVisitorConfig<'a> {
    type Error = SynError;

    fn try_from(value: &'a DeriveInput) -> Result<Self, Self::Error> {
        if let syn::Data::Enum(ref enum_variants) = value.data {
            let vis = &value.vis;
            let raw_enum_ident = &value.ident;
           
            Ok(EnumVisitorConfig {
                vis,
                raw_enum_ident,
                enum_variants,
            })
        } else {
            Err(SynError::new(value.ident.span(), "必须为枚举类型"))
        }
    }
}


fn generator_enum_visitor(enum_config: EnumVisitorConfig) -> TokenStream2 {
    let vis = enum_config.vis;
    let raw_enum_ident = enum_config.raw_enum_ident;
    let enum_ident = enum_config.const_values_ident();
    let variants = &enum_config.enum_variants.variants;

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
}

mod utils {
    pub fn append_str_ident(ident: &str, s: &str, span: proc_macro2::Span) -> syn::Ident {
        syn::Ident::new(format!("{}{}", ident, s).as_str(), span)
    }
}

mod model{
    use syn::parse::ParseStream;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Attrbute(Vec<(syn::Ident, syn::Lit)>);

    impl syn::parse::Parse for Attrbute {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            fn parse(input: ParseStream, out: &mut Vec<(syn::Ident, syn::Lit)>) -> syn::Result<()> {
                let ident: syn::Ident = input.parse()?;
                input.parse::<syn::Token![=]>()?;
                let lit: syn::Lit = input.parse()?;
                out.push((ident, lit));
                if input.parse::<syn::Token![,]>().is_ok() {
                    parse(input, out)
                } else {
                    Ok(())
                }
            }
            let mut v = vec![];
            parse(input, &mut v)?;
            Ok(Self(v))
        }
    }

    impl Attrbute {
        fn contain_ident(&self, t: &str) -> bool {
            self.0.iter().any(|(i, _)| i == t)
        }
        fn contain_idents(&self, ids: &[&str]) -> bool {
            self.0.iter().any(|(i, _)| ids.iter().any(|id| i == id))
        }
    }
}