// The macro invocation in the previous test case contained an empty loop body
// inside the braces. In reality we want for the macro to accept arbitrary
// tokens inside the braces.
//
// 上一个测试用例中的宏调用在大括号内包含一个空循环主体。
// 实际上，我们希望宏在大括号内接受任意令牌。
//
// The caller should be free to write whatever they want inside the braces. The
// seq macro won't care whether they write a statement, or a function, or a
// struct, or whatever else. So we will work with the loop body as a TokenStream
// rather than as a syntax tree.
//
// 调用者应该可以自由地在括号内写任何他们想写的东西。
// Seq宏不会关心他们是否编写语句、函数、结构或其他任何东西。
// 因此，我们将使用循环主体作为TokenStream，而不是语法树。
//
// Before moving on, ensure that your implementation knows what has been written
// inside the curly braces as a value of type TokenStream.
//
// 在继续之前，请确保您的实现知道在大括号中写入的内容作为TokenStream类型的值。
//
//
// Resources:
//
//   - Explanation of the purpose of proc-macro2:
//     https://docs.rs/proc-macro2/1.0/proc_macro2/

use seq::seq;

macro_rules! expand_to_nothing {
    ($arg:literal) => {
        // nothing
    };
}

seq!(N in 0..4 {
    expand_to_nothing!(N);
});

fn main() {}
/*
Seq {
    ident: Ident {
        ident: "N",
        span: #0 bytes(1351..1352),
    },
    in_token: In,
    lit_int: LitInt {
        token: 0,
    },
    until_token: DotDot,
    lit_int_2: LitInt {
        token: 4,
    },
    body: TokenStream [
        Group {
            delimiter: Brace,
            stream: TokenStream [
                Ident {
                    ident: "expand_to_nothing",
                    span: #0 bytes(1367..1384),
                },
                Punct {
                    ch: '!',
                    spacing: Alone,
                    span: #0 bytes(1384..1385),
                },
                Group {
                    delimiter: Parenthesis,
                    stream: TokenStream [
                        Ident {
                            ident: "N",
                            span: #0 bytes(1386..1387),
                        },
                    ],
                    span: #0 bytes(1385..1388),
                },
                Punct {
                    ch: ';',
                    spacing: Alone,
                    span: #0 bytes(1388..1389),
                },
            ],
            span: #0 bytes(1361..1391),
        },
    ],
}
*/