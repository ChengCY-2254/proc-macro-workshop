// One of the big things callers will want to do with the sequential indices N
// is use them as part of an identifier, like f0 f1 f2 etc.
//
// 调用者想用顺序索引N做的一件大事是将它们用作标识符的一部分，如f0 f1 f2等。
//
// Implement some logic to paste together any Ident followed by `~` followed by
// our loop variable into a single concatenated identifier.
//
// The invocation below will expand to:
//
// 实现一些逻辑，将任何标识符后跟`~`后跟我们的循环变量粘贴到单个连接标识符中。下面的调用将扩展到：
//
//     fn f1() -> u64 { 1 * 2 }
//     fn f2() -> u64 { 2 * 2 }
//     fn f3() -> u64 { 3 * 2 }
//
// Optionally, also support more flexible arrangements like `f~N~_suffix` ->
// f0_suffix f1_suffix etc, though the test suite only requires `prefix~N` so
// you will need to add your own tests for this feature.
//
//可选地，还支持更灵活的安排，
// 如`f~N~_suffix` -> f0_suffix f1_suffix等，
// 尽管测试只需要`prefix~N`，因此您需要为此功能添加自己的测试。
//
//
// Resources:
//
//     - Example of creating a new Ident from a string:
//       https://docs.rs/syn/2.0/syn/struct.Ident.html

use seq::seq;

seq!(N in 1..4 {
    fn f~N () -> u64 {
        N * 2
    }
});

// This f0 is written separately to detect whether your macro correctly starts
// with the first iteration at N=1 as specified in the invocation. If the macro
// incorrectly started at N=0 like in the previous tests cases, the first
// generated function would conflict with this one and the program would not
// compile.
fn f0() -> u64 {
    100
}

fn main() {
    let sum = f0() + f1() + f2() + f3();

    assert_eq!(sum, 100 + 2 + 4 + 6);
}
