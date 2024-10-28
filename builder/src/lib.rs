//! 为一个结构体配置Builder构建器
#![deny(unused)]
#![deny(unused_imports)]
#![deny(unused_mut)]
#![deny(unused_variables)]
#![deny(dead_code)]
#![deny(unused_extern_crates)]
#![deny(non_camel_case_types)]
#![deny(missing_docs)]
#![deny(unused_doc_comments)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[macro_use]
mod macros;
mod utils;

/// 为结构体生成`Builder`方法  
/// 例如
/// ```rust
/// use derive_builder::Builder;
///
/// #[derive(Builder)]
/// pub struct Command{
///     executable:String,
///     args:Vec<String>,
///     env:Vec<String>,
///     current_dir:String,
/// }
/// ```
/// 则生成
/// ```ignore
///# pub struct Command{
///#     executable:String,
///#     args:Vec<String>,
///#     env:Vec<String>,
///#     current_dir:String,
///# }
/// pub struct CommandBuilder {
///         executable: core::option::Option<String>,
///         args: core::option::Option<Vec<String>>,
///         env: core::option::Option<Vec<String>>,
///         current_dir: core::option::Option<String>,
///  }
/// impl Command {
///     pub fn builder() -> CommandBuilder {
///         CommandBuilder {
///             executable: None,
///             args: None,
///             env: None,
///             current_dir: None,
///       }         
///     }     
/// }
///
/// ```
///
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let config = unwrap!(BuilderConfig::try_from(&ast));
    // The ret is function result
    let mut ret = TokenStream2::new();
    let mut append = |f: fn(&BuilderConfig) -> Result<TokenStream2, syn::Error>| {
        f(&config)
            .unwrap_or_else(syn::Error::into_compile_error)
            .to_tokens(&mut ret)
    };
    // Generate the builder struct
    append(Generator::generate_builder_struct);
    // Generate the builder impl
    append(Generator::generate_builder_impl);
    // Generate the builder setter
    append(Generator::generate_builder_setter);
    // Generate the builder build func
    append(Generator::generate_builder_build);

    ret.into()
}

/// 所拿到的构建器配置
/// 所生成的结构体必须是具名字段
#[derive(Debug)]
#[allow(unused)]
struct BuilderConfig<'a> {
    /// 构建器名称
    name: &'a syn::Ident,
    /// 构建器访问属性
    vis: &'a syn::Visibility,
    /// 构建器字段
    fields: Vec<&'a syn::Field>,
    /// 构建器泛型
    generics: &'a syn::Generics,
}

impl<'a> TryFrom<&'a syn::DeriveInput> for BuilderConfig<'a> {
    type Error = syn::Error;

    /// 生成对[`syn::DeriveInput`]的解析结果
    fn try_from(input: &'a DeriveInput) -> Result<Self, Self::Error> {
        let name = &input.ident;
        let vis = &input.vis;
        let generics = &input.generics;
        let fields = if let syn::Data::Struct(ref data_struct) = input.data {
            data_struct.fields.iter().collect::<Vec<_>>()
        } else {
            return Err(Self::Error::new(name.span(), "目前仅支持具名结构体"));
        };

        Ok(Self {
            name,
            vis,
            fields,
            generics,
        })
    }
}

/// 结构体生成器，用于生成`Builder`各项内容
struct Generator;

impl Generator {
    /// 生成Builder结构体
    /// # 原结构体
    /// ```rust
    ///
    /// pub struct Command{
    ///     executable:Option<String>,
    ///     args:Vec<String>,
    ///     env:Vec<String>,
    ///     current_dir:String,
    /// }
    /// ```
    /// ---
    /// # 生成的Builder
    /// ```
    /// pub struct CommandBuilder {
    ///         executable: core::option::Option<String>,
    ///         args: core::option::Option<Vec<String>>,
    ///         env: core::option::Option<Vec<String>>,
    ///         current_dir: core::option::Option<String>,
    ///  }
    pub fn generate_builder_struct(config: &BuilderConfig) -> syn::Result<TokenStream2> {
        let struct_name = format_ident!("{}Builder", config.name);
        let vis = config.vis;
        let fields = config.fields.iter().map(|field| {
            let field_name = field.ident.as_ref();
            let ty = &field.ty;
            if utils::is_option(ty) {
                quote! {
                    #field_name: #ty,
                }
            } else {
                quote! {
                    #field_name: core::option::Option<#ty>,
                }
            }
        });
        Ok(quote! {
            #vis struct #struct_name{
                #(#fields)*
            }
        })
    }

    /// 生成Builder实现
    /// # 原结构体
    /// ```rust
    /// pub struct Command{
    ///     executable:String,
    ///     args:Vec<String>,
    ///     env:Vec<String>,
    ///     current_dir:String,
    /// }
    /// ```
    /// ---
    /// # 生成内容
    /// ```rust
    ///# pub struct Command{
    ///#      executable:String,
    ///#      args:Vec<String>,
    ///#      env:Vec<String>,
    ///#      current_dir:String,
    ///#  }
    ///pub struct CommandBuilder {
    ///        executable: core::option::Option<String>,
    ///        args: core::option::Option<Vec<String>>,
    ///        env: core::option::Option<Vec<String>>,
    ///        current_dir: core::option::Option<String>,
    ///  }
    ///impl Command {
    ///    pub fn builder() -> CommandBuilder {
    ///             CommandBuilder {
    ///                 executable: None,
    ///                 args: None,
    ///                 env: None,
    ///                 current_dir: None,
    ///        }
    ///     }
    /// }  
    /// ```
    pub fn generate_builder_impl(config: &BuilderConfig) -> syn::Result<TokenStream2> {
        let impl_struct_name = config.name;
        let builder_struct_name = format_ident!("{}Builder", config.name);
        let vis = config.vis;
        let fields = config.fields.iter().map(|f| {
            let field_name = f.ident.as_ref();
            quote! {
                #field_name: None,
            }
        });
        Ok(quote! {
            impl #impl_struct_name {
                #vis fn builder() -> #builder_struct_name {
                    #builder_struct_name {
                        #(#fields)*
                    }
                }
            }
        })
    }

    /// 生成Builder的setter方法
    /// # 生成内容
    /// ```rust
    ///# pub struct Command{
    ///#      executable:String,
    ///#      args:Vec<String>,
    ///#      env:Vec<String>,
    ///#      current_dir:String,
    ///#  }
    ///# pub struct CommandBuilder {
    ///#         executable: core::option::Option<String>,
    ///#         args: core::option::Option<Vec<String>>,
    ///#         env: core::option::Option<Vec<String>>,
    ///#         current_dir: core::option::Option<String>,
    ///#   }
    /// impl CommandBuilder{
    ///     fn executable(&mut self,executable:String)->&mut Self{
    ///       self.executable = Some(executable);
    ///       self
    ///     }
    /// }
    /// ```
    // #[allow(unused)]
    pub fn generate_builder_setter(config: &BuilderConfig) -> syn::Result<TokenStream2> {
        let impl_struct_name = format_ident!("{}Builder", config.name);
        let fields = config.fields.iter().map(|f| {
            let vis = config.vis;
            let fn_name = f.ident.as_ref();
            let field_name = f.ident.as_ref();
            // 如果是Option类型，就拿出内部类型，如果不是，就沿用类型
            let ty = {
                let ty = &f.ty;
                if utils::is_option(ty) {
                    utils::inner_type(ty)
                } else {
                    Some(ty)
                }
            };
            quote! {
                #vis fn #fn_name(&mut self,#field_name:#ty)->&mut Self{
                    self.#field_name = Some(#field_name);
                    self
                }
            }
        });
        Ok(quote! {
            impl #impl_struct_name {
                #(#fields)*
            }
        })
    }

    /// 生成`build`方法
    /// 需要判断字段是否是[`Option`]，如果是[`Option`]那么它的赋值方法将是
    /// ```ignore
    /// x:self.x.clone(),
    /// ```
    /// ---
    /// ```ignore
    /// impl CommandBuilder {
    ///       pub fn build(&self) -> core::result::Result<Command, Box<dyn core::error::Error>> {
    ///           Command {
    ///               executable: self.executable.clone().ok_or("struct Command not set field executable")?,
    ///               args: self.args.clone().ok_or("struct Command not set field args")?,
    ///               env: self.env.clone().ok_or("struct Command not set field env")?,
    ///               current_dir: self.current_dir.clone().ok_or("struct Command not set field current_dir")?,
    ///           }
    ///       }
    ///   }
    /// ```
    pub fn generate_builder_build(config: &BuilderConfig) -> syn::Result<TokenStream2> {
        let src_struct_name = config.name;
        let impl_struct_name = format_ident!("{}Builder", config.name);
        let vis = config.vis;

        let fields = config.fields.iter().map(|f| {
            let ident = f.ident.as_ref();
            let ty = &f.ty;
            let err_msg = format!(
                "struct {} not set field {}",
                src_struct_name,
                ident.unwrap()
            );
            // 判断字段是否是option
            if utils::is_option(ty) {
                quote! {
                    #ident: self.#ident.clone()
                }
            } else {
                quote! {
                    #ident: self.#ident.clone().ok_or(#err_msg)?,
                }
            }
        });

        Ok(quote! {
            impl #impl_struct_name {
                #vis fn build(&self)->core::result::Result<#src_struct_name,Box<dyn core::error::Error>>{
                    Ok(#src_struct_name{
                        #(#fields)*
                    })
                }
            }
        })
    }
}
