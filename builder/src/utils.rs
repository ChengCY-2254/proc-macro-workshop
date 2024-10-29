//! 存放一些快捷判断逻辑
#![allow(dead_code)]

use quote::format_ident;
use crate::config;

/// 判断是否是Option
///
/// # Arguments
///
/// * `f`: a [`syn::Type`]
///
/// returns: bool
#[inline]
pub fn is_option(f: &syn::Type) -> bool {
    is_type(f, "Option")
}

/// 获取一个泛型参数的内部类型
///
/// # Arguments
///
/// * `f`: a [`syn::Type`]
///
/// returns:[`Option<&syn::Type>`]
pub fn inner_type(f: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = f {
        // 长度是1，位置是0
        if path.segments.len() == 1 {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                ref args,
                ..
            }) = path.segments[0].arguments
            {
                if let Some(syn::GenericArgument::Type(ty)) = args.first() {
                    return Some(ty);
                }
            }
        }
    }
    None
}

/// 判断是否是某个包装类型，只能判断一级泛型，例如`Vec<String>`和`Option<String>`都会获得`String`类型
///
/// # Arguments
///
/// * `f`: 一个类型 [syn::Type]
/// * `ty_name`: 例如`Vec`、`Option`
///
/// returns: bool
#[inline]
pub fn is_type(f: &syn::Type, ty_name: &str) -> bool {
    if let syn::Type::Path(syn::TypePath { path, .. }) = f {
        // 长度是1，位置是0
        if path.segments.len() == 1 && path.segments[0].ident == ty_name {
            return true;
        }
    }
    false
}
/// 判断是否是Vec
///
/// # Arguments
///
/// * `f`: a [`syn::Type`]
///
/// returns: bool
#[inline]
pub fn is_vec(ty: &syn::Type) -> bool {
    is_type(ty, "Vec")
}

/// 判断一个标签中是否存在特定标签，如果存在，那么就取出为 `syn::LitStr`
///
/// # Arguments
///
/// * `attr` 是需要判断的标签
/// * `target_attr` 标签中的内部名称
///
/// returns: `Option<syn::LitStr>`
///
/// # Example
/// ```ignore
/// use syn::parse_quote;
/// let attrbute:syn::Attribute = parse_quote!(
///     #[builder(each = "arg")]
/// );
/// if let Some(lit_str)=unwrap_single_attribute(&attrbute,"each"){
/// ...
/// }
/// ```
pub fn unwrap_single_attribute(attr: &syn::Attribute, target_attr: &str) -> Option<syn::LitStr> {
    if let Ok(meta) = attr.parse_args::<syn::MetaNameValue>() {
        if meta.path.is_ident(target_attr) {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = meta.value
            {
                return Some(lit_str);
            }
        }
    }

    None
}

/// 获取一个`syn::MetaNameValue`attr的左值，用于打印日志
pub fn meta_name_value_attr_left_value(attr: &syn::Attribute) -> Option<syn::Ident> {
    if let Ok(ref meta) = attr.parse_args::<syn::MetaNameValue>() {
        return meta.path.get_ident().cloned();
    }
    None
}

/// 格式化一个将原标识符格式化为Builder构建器所需标识符
/// # Arguments
/// * `src_ident`: 目标结构体的原始名称 
///   
/// 原标识符  
/// `Command`  
/// 现标识符  
/// `CommandBuilder` 
#[inline]
pub fn get_builder_struct_ident(src_ident:&syn::Ident)->syn::Ident{
    format_ident!("{}{}",src_ident,config::BUILDER_SUFFIX)
}
