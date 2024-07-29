use proc_macro::TokenStream;
use syn::parse::ParseStream;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let ast: SeqParse = syn::parse_macro_input!(input as SeqParse);

    #[cfg(feature = "debug")]
    eprintln!("{:#?}", ast);

    let mut ret = proc_macro2::TokenStream::new();
    let buffer = syn::buffer::TokenBuffer::new2(ast.body.clone());
    
    // todo 需要重新构思
    // let (ret_1, expanded) = ast.find_block_to_expand_and_do_expand(buffer.begin());
    // if expanded {
    //     return ret_1.into();
    // }
    
    //没有匹配到#(xxx)*这种模式，直接展开
    for i in ast.start_int..ast.end_int {
        ret.extend(ast.expand(&ast.body, i))
    }

    ret.into()
}

#[derive(Debug)]
struct SeqParse {
    variable_ident: syn::Ident,
    start_int: usize,
    end_int: usize,
    body: proc_macro2::TokenStream,
}

impl syn::parse::Parse for SeqParse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let variable_ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let start_int: syn::LitInt = input.parse()?;
        input.parse::<syn::Token![..]>()?;
        let end_int: syn::LitInt = input.parse()?;

        let body_buf;
        syn::braced!(body_buf in input);
        let body: proc_macro2::TokenStream = body_buf.parse()?;

        Ok(Self {
            variable_ident,
            start_int: start_int.base10_parse()?,
            end_int: end_int.base10_parse()?,
            body,
        })
    }
}

impl SeqParse {
    fn expand(&self, ts: &proc_macro2::TokenStream, n: usize) -> proc_macro2::TokenStream {
        let mut ret = proc_macro2::TokenStream::new();
        let buf = ts.clone().into_iter().collect::<Vec<_>>();

        #[allow(clippy::needless_range_loop)]
        let mut idx = 0;
        while idx < buf.len() {
            let tt = &buf[idx];

            #[cfg(feature = "debug")]
            eprintln!("handle tt {},{:#?}", idx, tt);

            match tt {
                proc_macro2::TokenTree::Group(group) => {
                    let new_stream = self.expand(&group.stream(), n);
                    let wrap_in_group = proc_macro2::Group::new(group.delimiter(), new_stream);
                    ret.extend(quote::quote! {#wrap_in_group})
                }

                proc_macro2::TokenTree::Ident(ident) => {
                    if idx + 2 < buf.len() {
                        let next_tt = &buf[idx + 1];
                        if let proc_macro2::TokenTree::Punct(p) = next_tt {
                            if p.as_char() == '~' {
                                if let proc_macro2::TokenTree::Ident(ident2) = &buf[idx + 2] {
                                    if ident2 == &self.variable_ident {
                                        let new_ident_literal = format!("{}{}", ident, n);
                                        let new_ident = proc_macro2::Ident::new(
                                            &new_ident_literal,
                                            ident.span(),
                                        );
                                        ret.extend(quote::quote! {#new_ident});
                                        // 当前一个元素，使用了两个元素，合在一起就是三个元素
                                        idx += 3;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    // if this ident equals self.variable_ident then create new ident, type is
                    // Literal data is expanded func n param
                    if ident == &self.variable_ident.to_string() {
                        let new_ident = proc_macro2::Literal::i64_unsuffixed(n as i64);
                        ret.extend(quote::quote! {#new_ident});
                    } else {
                        ret.extend(quote::quote! {#ident});
                    }
                }

                _ => {
                    ret.extend(quote::quote! {#tt});
                }
            }
            idx += 1;
        }
        ret
    }
    /// 查找块中需要展开的部分，并返回被展开后的结果和是否已展开。
    fn find_block_to_expand_and_do_expand(
        &self,
        c: syn::buffer::Cursor,
    ) -> (proc_macro2::TokenStream, bool) {
        let mut found = false;
        let mut ret = proc_macro2::TokenStream::new();
        let mut cursor = c;
        // #(Irq~N,)*
        while !cursor.eof() {
            if let Some((punct_prefix, cursor_1)) = cursor.punct() {
                if punct_prefix.as_char() == '#' {
                    // 判断是否是一个被括号包裹的组
                    if let Some((group_cur, _, cursor_2)) =
                        cursor_1.group(proc_macro2::Delimiter::Parenthesis)
                    {
                        if let Some((punct_suffix, cursor_3)) = cursor_2.punct() {
                            if punct_suffix.as_char() == '*' {
                                for i in self.start_int..self.end_int {
                                    let t = self.expand(&group_cur.token_stream(), i);
                                    ret.extend(t);
                                }
                                cursor = cursor_3;
                                found = true;
                                continue;
                            }
                        }
                    }
                }
            }
            //之前没有找到或者规则不匹配的话，那么就在这里寻找括号中的内容，并将内容递归进行分析。
            if let Some((group_cur, _, next_cur)) =
                cursor.group(proc_macro2::Delimiter::Parenthesis)
            //()
            {
                let (t, f) = self.find_block_to_expand_and_do_expand(group_cur);
                found = f;
                ret.extend(quote::quote! {#t});
                cursor = next_cur;
                continue;
            } else if let Some((group_cur, _, next_cur)) =
                cursor.group(proc_macro2::Delimiter::Brace)
            //{}
            {
                let (t, f) = self.find_block_to_expand_and_do_expand(group_cur);
                found = f;
                ret.extend(quote::quote! {#t});
                cursor = next_cur;
                continue;
            } else if let Some((group_cur, _, next_cur)) =
                cursor.group(proc_macro2::Delimiter::Bracket)
            //[]
            {
                let (t, f) = self.find_block_to_expand_and_do_expand(group_cur);
                found = f;
                ret.extend(quote::quote! {#t});
                cursor = next_cur;
                continue;
            } else if let Some((punct, next_cur)) = cursor.punct() {
                //复制符号
                ret.extend(quote::quote! {#punct});
                cursor = next_cur;
                continue;
            } else if let Some((ident, next_cur)) = cursor.ident() {
                //复制标识符
                ret.extend(quote::quote! {#ident});
                cursor = next_cur;
                continue;
            } else if let Some((literal, next_cur)) = cursor.literal() {
                //复制字面量
                ret.extend(quote::quote! {#literal});
                cursor = next_cur;
                continue;
            }
        }

        (ret, found)
    }
}
