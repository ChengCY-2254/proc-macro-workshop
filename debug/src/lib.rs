use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, Error};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    do_expand(&input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn do_expand(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    eprintln!("{:#?}", ast);

    // let token_stream = dispatcher_input(ast);

    // Ok(token_stream.into())
    dispatcher_input(ast).map(|t| t.into())
}

/// 为结构体字段获取实现
fn dispatcher_input(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let fields_impl = get_fields_from_driver_input(input, |f| match f {
        syn::Fields::Named(syn::FieldsNamed { ref named, .. }) => Ok(field_named_impl(named)),
        syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => {
            Ok(field_unnamed_impl(unnamed))
        }
        _ => {
            return Err(syn::Error::new(input.span(), "struct fields check error"));
        }
    })
    .map_err(|_| syn::Error::new(input.span(), "Must define on as Struct, not Enum and Union"))?;
    // 存储字段的类型标识
    let mut field_type_names = vec![];
    // 存储某个特定类型的内部字段
    let mut phantomdata_type_param_names = vec![];
    get_fields_from_driver_input(input, |fields| {
        for field in fields {
            if let Some(s) = field_type_name(field) {
                field_type_names.push(s.clone());
            }
            if let Some(s) = ty_inner_type("PhantomData", &field.ty) {
                if let syn::Type::Path(ty) = s {
                    let ident = ty.path.segments[0].ident.clone();
                    phantomdata_type_param_names.push(ident);
                }
            }
        }
        Ok(())
    })?;
    let mut generics = input.generics.clone();
    for g in generics.params.iter_mut() {
        if let syn::GenericParam::Type(t) = g {
            let type_param_name = &t.ident;

            if phantomdata_type_param_names.contains(type_param_name)
                && !field_type_names.contains(&type_param_name)
            {
                continue;
            }
            t.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
    Ok(generate_trait(struct_name, fields_impl, generics))
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
         #(.field(#field_indexes_str, &format_args!(#field_format_strs,&self.#field_indexes)) )*
    }
}
/// 生成trait实现
fn generate_trait(
    struct_name: &syn::Ident,
    fields_impl: proc_macro2::TokenStream,
    generics: syn::Generics,
) -> proc_macro2::TokenStream {
    let struct_name_literal = struct_name.to_string();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics std::fmt::Debug for #struct_name #ty_generics #where_clause{
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
/// 获取一个类型的内部泛型，适用于单个泛型。
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
/// 获取字段的类型标识符
fn field_type_name(field: &syn::Field) -> Option<&syn::Ident> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field.ty
    {
        if segments.is_empty() {
            return None;
        }
        return Some(&segments[0].ident);
    }
    None
}

/// 获取结构体字段并进行操作
fn get_fields_from_driver_input<T, F>(driver_input: &syn::DeriveInput, mut f: F) -> syn::Result<T>
where
    F: FnMut(&syn::Fields) -> syn::Result<T>,
{
    if let syn::Data::Struct(syn::DataStruct { ref fields, .. }) = driver_input.data {
        return f(fields);
    }
    syn::Result::Err(Error::new(driver_input.span(), "Only Struct can use"))
}
