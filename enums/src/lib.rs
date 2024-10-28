use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Error as SynError};

/// 为枚举类型生成枚举值的常量
///```rust
/// #[derive(derive_enums::Enums)]
/// #[enums_set(fn_name="enums")]
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
/// #\[enums_set]标签
/// fn_name 可以自定义输出的方法名
#[proc_macro_derive(Enums, attributes(enums_set))]
pub fn enums(input: TokenStream) -> TokenStream {
    let mut ret = TokenStream2::new();
    let enum_input = parse_macro_input!(input as syn::DeriveInput);
    // eprintln!("{:#?}", enum_input);
    match EnumVisitorConfig::try_from(&enum_input) {
        Ok(config) => config
            .generator_func()
            .unwrap_or_else(SynError::into_compile_error)
            .to_tokens(&mut ret),
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
    /// 映射的方法名`#[enums_set(fn_name = "xxx")]`
    // pub fn_name: LazyCell<Option<String>>,
    pub literal_atters: Vec<model::LiteralAtters>,
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
    ///方法生成
    fn generator_func(&self) -> syn::Result<TokenStream2> {
        let enum_config = self;
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
        //获取方法名
        let fn_name = if let Some((fn_name, span)) =
            self.literal_atters.iter().map_while(|s| s.fn_name()).next()
        {
            syn::Ident::new(fn_name.as_str(), span)
        } else {
            syn::Ident::new("values", raw_enum_ident.span())
        };

        //生成一个const 常量并实现一个函数来返回这个常量
        Ok(quote! {
            #[doc(hidden)]
            const #enum_ident:&[&str] = &[#(#enum_values),*];
            impl #raw_enum_ident{
                #vis fn #fn_name()->&'static [&'static str]{
                    #enum_ident
                }
            }
        })
    }
}
impl<'a> TryFrom<&'a syn::DeriveInput> for EnumVisitorConfig<'a> {
    type Error = SynError;
    #[inline]
    fn try_from(value: &'a DeriveInput) -> Result<Self, Self::Error> {
        if let syn::Data::Enum(ref enum_variants) = value.data {
            let vis = &value.vis;
            let raw_enum_ident = &value.ident;
            let attrs = &value.attrs;
            let attrs = utils::get_attribute(attrs.iter(), "enums_set");
            let literal_atters = attrs
                .iter()
                .map_while(|attr| {
                    if let syn::Meta::List(syn::MetaList { ref tokens, .. }) = attr.meta {
                        let literal_atters =
                            syn::parse2::<model::LiteralAtters>(tokens.clone()).unwrap();
                        Some(literal_atters)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            Ok(EnumVisitorConfig {
                vis,
                raw_enum_ident,
                enum_variants,
                literal_atters,
            })
        } else {
            Err(SynError::new(value.ident.span(), "必须为枚举类型"))
        }
    }
}


mod utils {
    pub fn append_str_ident(ident: &str, s: &str, span: proc_macro2::Span) -> syn::Ident {
        syn::Ident::new(format!("{}{}", ident, s).as_str(), span)
    }
    /// 过滤标识符
    pub(crate) fn get_attribute<'a, Iter: Iterator<Item = &'a syn::Attribute>>(
        iter: Iter,
        ident: &str,
    ) -> Vec<syn::Attribute> {
        iter.filter(|a| {
            if a.path().is_ident(ident) {
                return true;
            }
            false
        })
            .map(Clone::clone)
            .collect()
    }
}

mod model {
    use syn::parse::ParseStream;

    #[derive(Eq, PartialEq, Clone)]
    pub(crate) struct LiteralAtter {
        pub name: syn::Ident,
        pub token: syn::Token![=],
        pub value: syn::Lit,
    }
    impl syn::parse::Parse for LiteralAtter {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let name: syn::Ident = input.parse()?;
            let token: syn::Token![=] = input.parse()?;
            let value: syn::Lit = input.parse()?;

            Ok(LiteralAtter { name, token, value })
        }
    }
    ///存储常量标签列表
    pub(crate) struct LiteralAtters(Vec<LiteralAtter>);
    impl LiteralAtters {
        #[inline]
        pub fn attr(&self, ident: &str) -> Option<&LiteralAtter> {
            self.0.iter().find(|id| id.name == ident)
        }
        pub fn fn_name(&self) -> Option<(String, proc_macro2::Span)> {
            let attr = self.attr("fn_name").map(|attr| {
                if let syn::Lit::Str(ref str) = attr.value {
                    Some((str.value(), attr.value.span()))
                } else {
                    None
                }
            });
            //打平
            attr.unwrap_or_default()
        }
    }
    impl std::ops::Deref for LiteralAtters {
        type Target = Vec<LiteralAtter>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl syn::parse::Parse for LiteralAtters {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let mut inner = vec![];
            inner.push(input.parse::<LiteralAtter>()?);
            while input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>().ok();
                //存在逗号分割属性
                let literal_attr = input.parse::<LiteralAtter>()?;
                inner.push(literal_attr);
            }
            Ok(LiteralAtters(inner))
        }
    }
}
