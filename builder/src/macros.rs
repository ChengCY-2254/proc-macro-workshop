//! 定义了一些宏  
//! [`unwrap!`] 用于在返回值为`proc::TokenStream`的主方法解包

#![allow(unused)]
/// 解包类型为 [`syn::Error`] 的错误，并将其错误通过[`proc_macro2::TokenStream`]返回。
macro_rules! unwrap {
        ($ret:ident,$t:expr) => {
            match $t.map_err(|e| e.to_compile_error()) {
                Ok(v) => v,
                Err(e) => {
                    $ret.extend(e);
                    return $ret.into();
                }
            }
        };
        ($t:expr) => {
            match $t.map_err(|e| e.to_compile_error()) {
                Ok(v) => v,
                Err(e) => {
                    let mut stream = proc_macro2::TokenStream::new();
                    stream.extend(e);
                    return stream.into();
                }
            }
        };
    }