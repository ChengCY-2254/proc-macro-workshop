use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    do_expand(&input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn do_expand(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    // eprintln!("{:#?}", ast);

    // let token_stream = dispatcher_input(ast);

    // Ok(token_stream.into())
    dispatcher_input(ast).map(|t| t.into())
}

/// 为结构体字段获取实现
fn dispatcher_input(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let fields_impl = if let syn::Data::Struct(syn::DataStruct { ref fields, .. }) = input.data {
        match fields {
            syn::Fields::Named(syn::FieldsNamed { ref named, .. }) => field_named_impl(named),
            syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => {
                field_unnamed_impl(unnamed)
            }
            _ => {
                return Err(syn::Error::new(input.span(), "struct fields check error"));
            }
        }
    } else {
        return Err(syn::Error::new(
            input.span(),
            "Must define on a Struct ,not Enum and Union",
        ));
    };
    Ok(generate_trait(struct_name, fields_impl))
}
/// 生成具名字段实现
fn field_named_impl(
    named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let field_impls = named.iter().map(|f| {
        let field_name = f.ident.clone().unwrap();
        let field_name_literal = field_name.to_string();
        let format_str = match get_custom_format_of_field(f, "debug") {
            Ok(Some(str)) => str.clone(),
            _ => "{:?}".to_string(),
        };
        quote! {
            //.field(#field_name_literal,&self.#field_name)
            .field(#field_name_literal,&format_args!(#format_str,&self.#field_name))
        }
    });
    quote! {
        #(#field_impls)*
    }
}
/// 生成未命名字段的实现
fn field_unnamed_impl(
    unnamed: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let field_indexes = (0..unnamed.len()).map(syn::Index::from);
    let field_indexes_str = (0..unnamed.len()).map(|idx| format!("{}", idx));
    let field_format_strs = unnamed
        .iter()
        .map(|f| match get_custom_format_of_field(f, "debug") {
            Ok(Some(str)) => str,
            _ => "{:?}".to_string(),
        });
    quote! {
        // #(.field(#field_indexes_str, &self.#field_indexes) )*
         #(.field(#field_indexes_str, &format_args!(#field_format_strs,&self.#field_indexes)) )*
    }
}
/// 生成trait实现
fn generate_trait(
    struct_name: &syn::Ident,
    fields_impl: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let struct_name_literal = struct_name.to_string();
    quote! {
        impl std::fmt::Debug for #struct_name{
            fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
                f.debug_struct(#struct_name_literal)
                #fields_impl
                .finish()
            }
        }
    }
}

/// 从字段中获取自定义属性的常量值
fn get_custom_format_of_field(field: &syn::Field, attribute: &str) -> syn::Result<Option<String>> {
    for attr in &field.attrs {
        match attr.meta {
            syn::Meta::NameValue(syn::MetaNameValue {
                ref path,
                ref value,
                ..
            }) if path.is_ident(attribute) => {
                return if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(str),
                    ..
                }) = value
                {
                    Ok(Some(str.value()))
                } else {
                    Ok(None)
                }
            }
            _ => { /*ignore*/ }
        }
    }
    Ok(None)
}
