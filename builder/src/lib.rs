use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input};

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

    let ty_is_option = |ty: &syn::Type| {
        if let syn::Type::Path(syn::TypePath { ref path, .. }) = ty {
            return path.segments.len() == 1 && path.segments[0].ident == "Option";
        }
        false
    };

    let unwrap_option_t = |ty: &syn::Type| -> syn::Ident {
        assert!(ty_is_option(ty));
        if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                ref args,
                ..
            }) = path.segments[0].arguments
            {
                if let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path, .. }, ..)) =
                    &args[0]
                {
                    return path.segments[0].ident.clone();
                }
            }
        };
        panic!("Option requires generic parameters")
    };

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
        if ty_is_option(ty) {
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
        if ty_is_option(ty) {
            let inner_ty = unwrap_option_t(ty);
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

    let build_fields = fields.iter().map(|f| {
        let ty = &f.ty;
        let name = &f.ident;
        if ty_is_option(ty) {
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
            pub fn build(&self) -> Result<#name, Box<(dyn std::error::Error + 'static) >> {
                Ok(#name{
                    #(#build_fields,)*
                })
            }

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
