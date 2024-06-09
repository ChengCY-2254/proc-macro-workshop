use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, Field, PathSegment};

/// from https://www.youtube.com/watch?v=geovSK3wMB8
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    //这是类型标识符
    let name = &ast.ident;
    //创建builder类型的新名字，span使用类型标识符的span，以表示是由它派生而来的。当它因原始名字产生错误的时候，它会指出是谁产生的错误。
    let builder_name = syn::Ident::new(&format!("{}Builder", &name), name.span());

    #[cfg(feature = "debug")]
    eprintln!("{:#?}", &ast);

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = &ast.data
    {
        named
    } else {
        panic!("Builder can only be derived for structs")
    };

    let optionized = fields.iter().map(|f| {
        let ty = &f.ty;
        let name = &f.ident;
        if ty_is_expect(ty, "Option") {
            quote! {
                #name: #ty
            }
        } else {
            quote! {
                #name: std::option::Option<#ty>
            }
        }
    });

    let methods = fields.iter().map(|f| {
        let ty = &f.ty;
        let name = &f.ident;
        if ty_is_expect(ty, "Option") {
            let inner_ty = ty_inner_type("Option", ty).expect("Option requires generic parameters");
            quote! {
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });

    let extend_methods = fields.iter().filter_map(|f| match extend_methods(f) {
        Some((true, _)) => None,
        Some((false, method)) => Some(method),
        None => None,
    });

    let build_fields = fields.iter().map(|f| {
        let ty = &f.ty;
        let name = &f.ident;
        if ty_is_expect(ty, "Option") {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name : self.#name.clone().ok_or(concat!("field ",stringify!(#name)," is not set"))?
            }
        }
    });
    // 构建builder方法字段，只需要获得字段名即可
    let builder_empty = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name : None
        }
    });
    // https://docs.rs/quote/latest/quote/macro.quote.html#interpolation
    let q = quote! {
        struct #builder_name{
            #(#optionized,)*
        }
        impl #builder_name{
            #(#methods)*
            pub fn build(&self) -> std::result::Result<#name,std::boxed::Box<(dyn std::error::Error + 'static) >> {
                Ok(#name{
                    #(#build_fields,)*
                })
            }
            #(#extend_methods)*

        }
        impl #name{
            fn builder()-> #builder_name{
                #builder_name{
                    #(#builder_empty,)*
                }
            }
        }
    };
    q.into()
}
/// 获得一个类型中的泛型
/// ```json
/// ty: Type::Path {
///     qself: None,
///     path: Path {
///         leading_colon: None,
///         segments: [
///             PathSegment {
///                 ident: Ident {
///                     ident: "Vec",
///                     span: #0 bytes(1045..1048),
///                 },
///                 arguments: PathArguments::AngleBracketed {
///                     colon2_token: None,
///                     lt_token: Lt,
///                     args: [
///                         GenericArgument::Type(
///                             Type::Path {
///                                 qself: None,
///                                 path: Path {
///                                     leading_colon: None,
///                                     segments: [
///                                         PathSegment {
///                                             ident: Ident {
///                                                 ident: "String",
///                                                 span: #0 bytes(1049..1055),
///                                             },
///                                             arguments: PathArguments::None,
///                                         },
///                                     ],
///                                 },
///                             },
///                         ),
///                     ],
///                     gt_token: Gt,
///                 },
///             },
///         ],
///  },
///},
///
///
///```
fn ty_inner_type<'a>(wrapper: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper {
            return None;
        }
        if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
            if inner_ty.args.len() != 1 {
                return None;
            }

            let inner_ty = inner_ty.args.first().unwrap();
            if let syn::GenericArgument::Type(ref t) = inner_ty {
                return Some(t);
            }
        }
    }
    None
}

fn ty_is_expect(ty: &syn::Type, type_str: &str) -> bool {
    if let syn::Type::Path(syn::TypePath { ref path, .. }) = ty {
        return path.segments.len() == 1 && path.segments[0].ident == type_str;
    }
    false
}
/// 解包参数为Vec的字段类型,将其列表中的类型做成一个方法
/// 例如下面代码示例，就可以通过标签将env中的参数单独解包出来，成为一个方法，并提供多次调用，依次往其内部添加值
/// ```rust
/// use derive_builder::Builder;
/// #[derive(Builder)]
/// struct Command{
///     #[builder(each="e")]
///     env:Vec<String>
/// }
///
/// let command = Command::builder().e("a".to_owned()).e("b".to_owned()).build();
/// ```
fn extend_methods(f: &Field) -> Option<(bool, proc_macro2::TokenStream)> {
    let mut avoid_conflict = false;
    let name = &f.ident;
    #[cfg(feature = "debug")]
    if !f.attrs.is_empty() {
        eprintln!("{:#?}", f.attrs)
    }

    if f.attrs.is_empty() {
        #[cfg(feature = "debug")]
        eprintln!("attr is none");
        return None;
    }

    if let Some((_path, tokens)) = take_field_attr(f, "builder") {
        if let Some(
            GeneralAttr {
                ident,
                punct,
                literal,
            },
            ..,
        ) = GeneralAttr::parse_from_token_stream(tokens)
        {
            if ident != "each" {
                panic!(r#"expected `builder(each = "...")`"#)
            }
            if punct.as_char() != '=' {
                panic!("expected '=', found {}", punct)
            }
            match literal {
                syn::Lit::Str(s) => {
                    let inner_ty = ty_inner_type("Vec", &f.ty);
                    let arg = syn::Ident::new(&s.value(), s.span());
                    if arg == f.ident.clone().expect("Named fields are required") {
                        avoid_conflict = true;
                    }
                    return Some((
                        avoid_conflict,
                        quote! {
                            pub fn #arg(&mut self,#arg:#inner_ty)->&mut Self{
                                match self.#name{
                                    Some(ref mut v)=>{
                                        v.push(#arg);
                                    },
                                    None=>{
                                        self.#name = Some(vec![#arg])
                                    }
                                };
                                self
                            }
                        },
                    ));
                }
                literal => panic!("expected literal, found {:?}", literal),
            }
        }
    }

    None
}

/// 尝试从field中取出指定标签和TokenStream
fn take_field_attr<'a>(
    f: &'a Field,
    expect_attr: &str,
) -> Option<(&'a PathSegment, &'a proc_macro2::TokenStream)> {
    for attr in &f.attrs {
        if let syn::Meta::List(syn::MetaList { path, tokens, .. }) = &attr.meta {
            if path.segments.is_empty() {
                return None;
            }
            for segment in &path.segments {
                if segment.ident == expect_attr {
                    return Some((segment, tokens));
                }
            }
        }
    }
    None
}

struct GeneralAttr {
    ident: syn::Ident,
    punct: proc_macro2::Punct,
    literal: syn::Lit,
}

impl syn::parse::Parse for GeneralAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.call(syn::Ident::parse)?;
        let punct: proc_macro2::Punct = input.call(proc_macro2::Punct::parse)?;
        let literal: syn::Lit = input.call(syn::Lit::parse)?;
        Ok(Self {
            ident,
            punct,
            literal,
        })
    }
}

impl GeneralAttr {
    fn parse_from_token_stream(token_stream: &proc_macro2::TokenStream) -> Option<Self> {
        let mut tt = token_stream.clone().into_iter();

        let ident = match tt.next().unwrap() {
            proc_macro2::TokenTree::Ident(ident) => ident,
            tt => panic!("expeted '{}',but found {}", "Ident", tt),
        };

        let punct = match tt.next().unwrap() {
            proc_macro2::TokenTree::Punct(punct) => punct,
            tt => panic!("expected '{}',but fount {}", "punct", tt),
        };

        let literal = match tt.next().expect("expected str") {
            proc_macro2::TokenTree::Literal(literal) => syn::Lit::new(literal),
            tt => panic!("expected string, but found {}", tt),
        };

        Some(Self {
            ident,
            punct,
            literal,
        })
    }
}
