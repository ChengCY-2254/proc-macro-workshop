use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    return match do_expand(&input) {
        Ok(token_stream) => token_stream,
        Err(e) => e.to_compile_error().into(),
    };
}

fn do_expand(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    // eprintln!("{:#?}", ast);

    let token_stream = dispatcher_input(ast);

    Ok(token_stream.into())
}

/// 为结构体字段获取实现
fn dispatcher_input(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = &input.ident;
    let fields_impl = if let syn::Data::Struct(syn::DataStruct { ref fields, .. }) = input.data {
        match fields {
            syn::Fields::Named(syn::FieldsNamed { ref named, .. }) => field_named_impl(named),
            syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => {
                field_unnamed_impl(unnamed)
            }
            _ => {
                panic!("empty fields")
            }
        }
    } else {
        panic!("Must define on a Struct ,not Enum and Union")
    };
    generate_trait(struct_name, fields_impl)
}

fn field_named_impl(
    named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Option<proc_macro2::TokenStream> {
    let field_impls = named.iter().map(|f| {
        let field_name = f.ident.clone().unwrap();
        let field_name_literal = field_name.to_string();
        quote! {
            .field(#field_name_literal,&self.#field_name)
        }
    });
    Some(quote! {
        #(#field_impls)*
    })
}

fn field_unnamed_impl(
    unnamed: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Option<proc_macro2::TokenStream> {
    let field_indexes = (0..unnamed.len()).map(syn::Index::from);
    let field_indexes_str = (0..unnamed.len()).map(|idx| format!("{}", idx));
    Some(quote! {
        #(.field(#field_indexes_str, &self.#field_indexes) )*
    })
}
/// 生成trait实现
fn generate_trait(
    struct_name: &syn::Ident,
    fields_impl: Option<proc_macro2::TokenStream>,
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
