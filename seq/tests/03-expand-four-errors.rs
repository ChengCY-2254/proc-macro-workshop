// Now construct the generated code! Produce the output TokenStream by repeating
// the loop body the correct number of times as specified by the loop bounds and
// replacing the specified identifier with the loop counter.
//
// The invocation below will need to expand to a TokenStream containing:
//
// 现在构建生成的代码！
// 通过重复循环主体指定的循环边界指定的正确次数，
// 并将指定的标识符替换为循环计数器，
// 从而生成输出TokenStream。
// 下面的调用需要扩展到包含以下内容的TokenStream：
//
//     compile_error!(concat!("error number ", stringify!(0)));
//     compile_error!(concat!("error number ", stringify!(1)));
//     compile_error!(concat!("error number ", stringify!(2)));
//     compile_error!(concat!("error number ", stringify!(3)));
//
// This test is written as a compile_fail test because our macro isn't yet
// powerful enough to do anything useful. For example if we made it generate
// something like a function, every one of those functions would have the same
// name and the program would not compile.
//
// 这个测试是作为compile_fail测试编写的因为我们的宏还不够强大,无法做什么有用的事情。
// 例如，如果我们让它生成类似函数的东西，
// 这些函数中的每一个都将具有相同的名称，
// 程序将不会编译。

use seq::seq;

seq!(N in 0..4 {
    compile_error!(concat!("error number ", stringify!(N)));
});

fn main() {}
