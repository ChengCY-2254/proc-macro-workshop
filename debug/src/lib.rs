use proc_macro::TokenStream;
use std::collections::HashMap;

use quote::quote;
use syn::{Error, parse_macro_input, parse_quote, spanned::Spanned, visit::Visit};

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
    // 解析字段
    let struct_name = &input.ident;
    let fields_impl = get_fields_from_driver_input(input, |f| match f {
        syn::Fields::Named(syn::FieldsNamed { ref named, .. }) => Ok(field_named_impl(named)),
        syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => {
            Ok(field_unnamed_impl(unnamed))
        }
        _ => {
             Err(syn::Error::new(input.span(), "struct fields check error"))
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
            if let Some(syn::Type::Path(ty)) = ty_inner_type("PhantomData", &field.ty) {
                    let ident = ty.path.segments[0].ident.clone();
                    phantomdata_type_param_names.push(ident);
            }
        }
        Ok(())
    })?;
    // 获得关联类型
    let associated_types_map = get_generic_associated_types(input);
    let mut generics = input.generics.clone();
    if let Ok(Some(hatch)) = get_struct_custom_attrbute_str(input, "bound") {
        generics
            .make_where_clause()
            .predicates
            .push(syn::parse_str(hatch.as_ref()).unwrap());
    } else {
        for g in generics.params.iter_mut() {
            if let syn::GenericParam::Type(t) = g {
                let type_param_name = &t.ident;

                if phantomdata_type_param_names.contains(type_param_name)
                    && !field_type_names.contains(type_param_name)
                {
                    continue;
                }

                if associated_types_map.contains_key(type_param_name)
                    && !field_type_names.contains(type_param_name)
                {
                    continue;
                }
                t.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
        // 处理where子句
        generics.make_where_clause();
        for (_, associated_types) in associated_types_map {
            for associated_type in associated_types {
                generics
                    .where_clause
                    .as_mut()
                    .unwrap()
                    .predicates
                    .push(parse_quote!(#associated_type:std::fmt::Debug))
            }
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
        let format_str = match get_custom_attribute_value(f, "debug") {
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
        .map(|f| match get_custom_attribute_value(f, "debug") {
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
fn get_custom_attribute_value(field: &syn::Field, attribute: &str) -> syn::Result<Option<String>> {
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

/// 从结构体中通过标签名获取这个标签的单个值
/// #[declare(attribute = "")]
fn get_struct_custom_attrbute_str(
    input: &syn::DeriveInput,
    attribute: &str,
) -> syn::Result<Option<String>> {
    let mut ret = None;
    for attr in &input.attrs {
        match attr.meta {
            syn::Meta::List(syn::MetaList {
                ref path,
                ref tokens,
                ..
            }) => {
                if path.is_ident("debug") {
                    let tokens = tokens.clone().into();
                    let attr: Attribute = syn::parse(tokens)?;
                    if attr.ident != attribute {
                        continue;
                    }
                    if let syn::Lit::Str(litstr) = attr.literal {
                        ret = Some(litstr.value());
                        return Ok(ret);
                    }
                }
            }

            _ => { /*ignore*/ }
        }
    }
    Ok(ret)
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

struct TypePathVisitor {
    // 筛选条件
    generic_type_names: Vec<syn::Ident>,
    // 通过筛选条件获得的路径
    associated_types: HashMap<syn::Ident, Vec<syn::TypePath>>,
}

impl<'ast> syn::visit::Visit<'ast> for TypePathVisitor {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if node.path.segments.len() >= 2 {
            let generic_type_name = node.path.segments[0].ident.clone();
            if self.generic_type_names.contains(&generic_type_name) {
                self.associated_types
                    .entry(generic_type_name)
                    .or_default()
                    .push(node.clone())
            }
        }
        syn::visit::visit_type_path(self, node)
    }
}

fn get_generic_associated_types(
    input: &syn::DeriveInput,
) -> HashMap<syn::Ident, Vec<syn::TypePath>> {
    //从结构体中获取泛型列表
    let generic_type_names: Vec<syn::Ident> = input
        .generics
        .params
        .iter()
        .filter_map(|f| {
            if let syn::GenericParam::Type(ty) = f {
                return Some(ty.ident.clone());
            }
            None
        })
        .collect();
    //查询结构体中的泛型路径
    let mut visitor = TypePathVisitor {
        generic_type_names,
        associated_types: HashMap::new(),
    };
    visitor.visit_derive_input(input);
    visitor.associated_types
}

struct Attribute {
    ident: syn::Ident,
    punct: syn::Token![=],
    literal: syn::Lit,
}

impl syn::parse::Parse for Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident:input.parse()?,
            punct:input.parse()?,
            literal:input.parse()?,
        })
    }
}
