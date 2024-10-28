//! 存放一些快捷判断逻辑
#![allow(dead_code)]

/// 判断是否是Option
///
/// # Arguments
///
/// * `f`: a [`syn::Type`]
///
/// returns: bool
#[inline]
pub fn is_option(f: &syn::Type) -> bool {
    is_type(f,"Option")
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
