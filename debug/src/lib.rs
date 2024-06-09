use proc_macro::TokenStream;
use quote::quote;
use syn::MetaNameValue;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    eprint!("{:#?}", ast);
    let name = &ast.ident;
    if let Some(fields) = take_struct_fields(&ast) {
        let write_fn = fields.iter().map(|f| {
            let name = &f.ident;
            if let Some(syn::Attribute { meta, .. }) = check_attribute(f, "debug") {
                if let syn::Meta::NameValue(MetaNameValue {
                    path: _path,
                    value: syn::Expr::Lit(literal),
                    ..
                }) = meta
                {
                    if let syn::Lit::Str(show_expressions) = &literal.lit {
                        let template = " {}: [mark],".to_string();

                        let mut st = show_expressions.token().to_string();
                        st.remove(0);
                        st.remove(st.len() - 1);
                        let template_format = template.replace("[mark]", &st);
                        return quote! {
                            text.push_str(
                                &format!(#template_format,stringify!(#name),self.#name)
                            );
                        };
                    }
                }
            }
            quote! {
                text.push_str(
                    &format!(" {}: \"{}\",",stringify!(#name),self.#name)
                );
            }
        });
        return quote! {
            impl std::fmt::Debug for #name{
                #[inline]
                fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
                    let mut text = String::new();
                    write!(f,"{} {{",stringify!(#name))?;
                    #(#write_fn)*
                    let _ =text.pop();

                    text.push(' ');
                    write!(f,"{}",text)?;
                    write!(f,"}}")?;
                    Ok(())
                }
            }
        }
        .into();
    }

    panic!("Error Only used for Struct")
}

fn take_struct_fields(ast: &syn::DeriveInput) -> Option<&syn::Fields> {
    if let syn::Data::Struct(ref s) = ast.data {
        return Some(&s.fields);
    }
    None
}

// struct A {}

// impl std::fmt::Debug for A {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut text = format!("abc{}", 1);
//         text.pop
//     }
// }

/// 解析标签内容 格式为 #[ident punct literal]
///                  #[debug = "0b{:08b}"]
// struct LiteralAttribute {
//     ident: syn::Ident,
//     punct: proc_macro2::Punct,
//     literal: syn::Lit,
// }

// impl syn::parse::Parse for LiteralAttribute {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let ident: syn::Ident = input.call(syn::Ident::parse)?;
//         let punct: proc_macro2::Punct = input.call(proc_macro2::Punct::parse)?;
//         let literal: syn::Lit = input.call(syn::Lit::parse)?;
//         Ok(Self {
//             ident,
//             punct,
//             literal,
//         })
//     }
// }

// impl LiteralAttribute {
//     fn parse_from_token_stream(token_stream: &proc_macro2::TokenStream) -> Option<Self> {
//         let mut tt = token_stream.clone().into_iter();

//         let ident = match tt.next().unwrap() {
//             proc_macro2::TokenTree::Ident(ident) => ident,
//             tt => panic!("expeted '{}',but found {}", "Ident", tt),
//         };

//         let punct = match tt.next().unwrap() {
//             proc_macro2::TokenTree::Punct(punct) => punct,
//             tt => panic!("expected '{}',but fount {}", "punct", tt),
//         };

//         let literal = match tt.next().expect("expected str") {
//             proc_macro2::TokenTree::Literal(literal) => syn::Lit::new(literal),
//             tt => panic!("expected string, but found {}", tt),
//         };

//         Some(Self {
//             ident,
//             punct,
//             literal,
//         })
//     }
// }
/// 通过字段和一个标签来判断该字段上有没有这么一个标签，如果有，则返回该标签的ast节点，如果没有，就返回None
fn check_attribute<'a>(f: &'a syn::Field, attr_str: &str) -> Option<&'a syn::Attribute> {
    for attribute in f.attrs.iter() {
        if let syn::Meta::NameValue(ref meta) = attribute.meta {
            if meta.path.segments.is_empty() {
                return None;
            }
            if &meta.path.segments[0].ident == attr_str {
                return Some(attribute);
            }
        }
    }
    None
}
