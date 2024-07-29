use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use std::convert::AsRef;
use std::ops::Deref;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, DeriveInput, Error as SynError, Field};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut ret = TokenStream2::new();
    let ast = parse_macro_input!(input as syn::DeriveInput);
    // eprintln!("{:#?}", ast);

    if let syn::Data::Struct(_) = &ast.data {
        //生成builder结构体
        generated_builder_struct_constructor(&ast)
            .unwrap_or_else(SynError::into_compile_error)
            .to_tokens(&mut ret);
        //生成builder结构体的set方法
        generated_builder_struct_method(&ast)
            .unwrap_or_else(SynError::into_compile_error)
            .to_tokens(&mut ret);
        //生成builder结构体的build方法
        generated_builder_struct_build(&ast)
            .unwrap_or_else(SynError::into_compile_error)
            .to_tokens(&mut ret);
        //生成宿主结构体的Builder入口实现
        generated_builder_impl(&ast)
            .unwrap_or_else(SynError::into_compile_error)
            .to_tokens(&mut ret);
    } else {
        return SynError::new(ast.span(), "Builder必须使用在结构体上")
            .into_compile_error()
            .into();
    };

    ret.into()
}

/// 生成Builder结构体的构造函数
fn generated_builder_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let impl_struct_name = input.ident.clone();

    let out_struct_name = append_str_ident(&input.ident, "Builder");

    let init_struct_fields = if let syn::Data::Struct(ref data) = input.data {
        data.fields.iter().map(|f| {
            let field_name = &f.ident;
            quote! {
                #field_name:None
            }
        })
    } else {
        return Err(SynError::new(input.span(), "Builder必须使用在结构体上"));
    };
    Ok(quote! {
        impl #impl_struct_name {
            fn builder()->#out_struct_name{
                #out_struct_name{
                    #(#init_struct_fields,)*
                }
            }
        }
    })
}
/// 生成Builder结构体的设置方法
fn generated_builder_struct_method(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let impl_struct = append_str_ident(&input.ident, "Builder");
    let struct_fields = if let syn::Data::Struct(ref data) = input.data {
        data.fields.iter().map(|f| {
            let mut each = false;
            let raw_field_name = f.ident.clone().unwrap();
            let mut each_field_name = None;
            let mut ty = &f.ty;
            //检查是否有builder标签
            let attr: Vec<_> = get_attribute(f, "builder");

            if attr.not_empty() {
                if let syn::Meta::List(syn::MetaList { ref tokens, .. }) = attr[0].meta {
                    let attrbutes: MyAttribute = syn::parse2(tokens.clone()).unwrap();
                    if attrbutes.is_empty() {
                        return Err(SynError::new(f.span(), "builder标签必须有属性"));
                    }
                    if !attrbutes.contain_idents(vec!["each"].as_ref()) {
                        let (ident, _) = attrbutes.last().unwrap();
                        return Err(SynError::new(
                            ident.span(),
                            format!("builder标签必须有each属性,当前有一个未知属性{}", ident),
                        ));
                    }
                    for (ident, lit) in attrbutes.iter() {
                        if ident == "each" {
                            if let syn::Lit::Str(lit) = lit {
                                each = true;
                                each_field_name =
                                    Some(syn::Ident::new(lit.value().as_str(), lit.span()));
                                // 默认其类型为Vec，取出内部的类型覆盖掉当前类型
                                if is_ty_wapper(ty, "Vec") {
                                    ty = ty_inner_type(ty).unwrap()
                                } else {
                                    return Err(SynError::new(f.span(), "each标签的属性必须为Vec"));
                                }
                            } else {
                                return Err(SynError::new(
                                    f.span(),
                                    "builder标签的each属性必须为字符串",
                                ));
                            }
                        }
                    }
                }
            }
            if is_ty_wapper(ty, "Option") {
                ty = ty_inner_type(ty).unwrap();
                Ok(quote! {
                    fn #raw_field_name(&mut self,#raw_field_name:#ty)->&mut Self{
                        self.#raw_field_name=Some(#raw_field_name);
                        self
                    }
                })
            } else if each {
                Ok(quote! {
                    fn #each_field_name(&mut self,#each_field_name:#ty)->&mut Self{
                        match self.#raw_field_name {
                            Some(ref mut v)=>v.push(#each_field_name),
                            None=>self.#raw_field_name = Some(vec![#each_field_name])
                        };
                        self
                    }
                })
            } else {
                Ok(quote! {
                    fn #raw_field_name(&mut self,#raw_field_name:#ty)->&mut Self{
                        self.#raw_field_name=Some(#raw_field_name);
                        self
                    }
                })
            }
        })
    } else {
        return Err(SynError::new(
            input.span(),
            "Builder can only be derived for structs",
        ));
    };
    let struct_fields = struct_fields.map(|f| f.unwrap_or_else(SynError::into_compile_error));
    Ok(quote! {
        impl #impl_struct {
            #(#struct_fields )*
        }
    })
}

/// 生成Builder结构体的build方法
fn generated_builder_struct_build(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = append_str_ident(&input.ident, "Builder");
    let raw_struct = input.ident.clone();
    let struct_fields = if let syn::Data::Struct(ref data) = input.data {
        data.fields.iter().map(|f| {
            let field_name = &f.ident.clone().unwrap();
            let mut each = false;
            let attr: Vec<_> = get_attribute(f,"builder");
            if !attr.is_empty() {
                if let syn::Meta::List(syn::MetaList{ref tokens,..}) = attr[0].meta {
                    let my_attr:syn::Result<MyAttribute> = syn::parse2(tokens.clone());
                    if my_attr.is_ok() && my_attr.unwrap().contain_ident("each") {
                        each=true;
                    }
                }
            }
            if is_ty_wapper(&f.ty, "Option") {
                quote! {
                    #field_name: self.#field_name.clone()
                }
            }else if each {
                quote!{
                    #field_name: self.#field_name.clone().unwrap_or_default()
                }
            } else {
                quote! {
                    #field_name: self.#field_name.clone().ok_or(concat!("field ",stringify!(#field_name)," is not set"))?
                }
            }
        })
    } else {
        return Err(SynError::new(input.span(), "Builder必须使用在结构体上"));
    };

    Ok(quote! {
        impl #struct_name {
            pub fn build(&self)->core::result::Result<#raw_struct,std::boxed::Box<dyn std::error::Error + 'static>>{
                Ok(
                    #raw_struct{
                        #(#struct_fields,)*
                    }
                )
            }
        }
    })
}
/// 从字段中获取标签
fn get_attribute<'a>(f: &'a Field, ident: &str) -> Vec<&'a Attribute> {
    f.attrs
        .iter()
        .filter(|a| {
            if a.path().is_ident(ident) {
                return true;
            }
            false
        })
        .collect()
}

/// 生成Builder结构体
fn generated_builder_struct_constructor(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = append_str_ident(&input.ident, "Builder");
    let struct_vis = input.vis.clone();
    let struct_fields = if let syn::Data::Struct(ref data) = input.data {
        data.fields.iter().map(|f| {
            let field_name = &f.ident;
            let field_ty = &f.ty;
            if is_ty_wapper(field_ty, "Option") {
                quote! {
                    #field_name:#field_ty
                }
            } else {
                quote! {
                    #field_name:std::option::Option<#field_ty>
                }
            }
        })
    } else {
        return Err(SynError::new(
            input.span(),
            "Builder can only be derived for structs",
        ));
    };
    Ok(quote! {
        #struct_vis struct #struct_name {
            #(#struct_fields,)*
        }
    })
}

/// 向一个标识符后面添加内容
fn append_str_ident(ident: &syn::Ident, s: &str) -> syn::Ident {
    syn::Ident::new(format!("{}{}", ident, s).as_str(), ident.span())
}

/// 判断是否是某个类型
fn is_ty_wapper(ty: &syn::Type, ty_str: &str) -> bool {
    if let syn::Type::Path(p) = ty {
        if p.path.segments.is_empty() {
            return false;
        }
        if let Some(path_segement) = p.path.segments.last() {
            if path_segement.ident == ty_str {
                return true;
            }
        }
    }
    false
}

/// 返回一个泛型的内部类型
#[allow(unused)]
fn ty_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(p) = ty {
        if p.path.segments.is_empty() {
            return None;
        }

        if let Some(path_segement) = p.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                ref args,
                ..
            }) = path_segement.arguments
            {
                return if let Some(syn::GenericArgument::Type(ty)) = args.last() {
                    Some(ty)
                } else {
                    None
                };
            }
        }
    }
    None
}

/// 将 #[xxx(foo="bar",ignore=true,...)]之类的标签解析掉,只关注标签中的值，不关心标签的名字
/// ``` json Attribute {
///     attrs: [
///         (
///             Ident {
///                 ident: "each",
///                 span: #0 bytes(2328..2332),
///             },
///             Lit::Str {
///                 token: "arg",
///             },
///         ),
///         (
///             Ident {
///                 ident: "ignore",
///                 span: #0 bytes(2341..2347),
///             },
///             Lit::Bool {
///                 value: true,
///             },
///         ),
///     ],
/// }
/// ```
#[derive(Debug)]
struct MyAttribute(Vec<(syn::Ident, syn::Lit)>);

impl Deref for MyAttribute {
    type Target = Vec<(syn::Ident, syn::Lit)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[allow(dead_code)]
impl MyAttribute {
    fn contain_ident(&self, t: &str) -> bool {
        self.iter().any(|(i, _)| i == t)
    }
    fn contain_idents(&self, ids: &[&str]) -> bool {
        self.iter().any(|(i, _)| ids.iter().any(|id| i == id))
    }
    fn for_each(&self, f: impl Fn(&syn::Ident, &syn::Lit)) {
        if self.is_empty() {
            return;
        }
        for (ident, lit) in self.iter() {
            f(ident, lit)
        }
    }
    fn get(&self, id: &str) -> Option<&syn::Lit> {
        for (ident, lit) in self.iter() {
            if ident == id {
                return Some(lit);
            }
        }
        None
    }
}

impl syn::parse::Parse for MyAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        fn parse(
            input: ParseStream,
            out: Vec<(syn::Ident, syn::Lit)>,
        ) -> syn::Result<Vec<(syn::Ident, syn::Lit)>> {
            let mut out = out;
            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let lit: syn::Lit = input.parse()?;
            out.push((ident, lit));
            //如果有逗号分割
            if input.parse::<syn::Token![,]>().is_ok() {
                parse(input, out)
            } else {
                Ok(out)
            }
        }
        let v = vec![];

        Ok(Self(parse(input, v)?))
    }
}
trait VecExt {
    fn not_empty(&self) -> bool;
}
impl<E> VecExt for Vec<E> {
    fn not_empty(&self) -> bool {
        !self.is_empty()
    }
}
impl<'a, E> VecExt for &'a [E] {
    fn not_empty(&self) -> bool {
        !self.is_empty()
    }
}
