// So far our macro has repeated the entire loop body. This is not sufficient
// for some use cases because there are restrictions on the syntactic position
// that macro invocations can appear in. For example the Rust grammar would not
// allow a caller to write:
//
// 到目前为止，我们的宏已经重复了整个循环体。
// 对于某些用例来说，这还不够，因为宏调用可以出现的语法位置受到限制。
// 例如，Rust语法不允许调用者写：
//
//     enum Interrupt {
//         seq!(N in 0..16 {
//             Irq~N,
//         });
//     }
//
// because this is just not a legal place to put a macro call.
// 因为这不是一个进行宏调用的地方。
// Instead we will implement a way for the caller to designate a specific part
// of the macro input to be repeated, so that anything outside that part does
// not get repeated. The repeated part will be written surrounded by #(...)*.
//
// 相反，我们将实现一种方法，让调用者指定要重复的宏输入的特定部分，
// 这样该部分以外的任何内容都不会重复。重复的部分将由 #(...)* 包含。
//
// The invocation below should expand to:
// 下面的调用应该扩展到：
//
//     #[derive(Copy, Clone, PartialEq, Debug)]
//     enum Interrupt {
//         Irq0,
//         ...
//         Irq15,
//     }
//
// Optionally, allow for there to be multiple separate #(...)* sections,
// although the test suite does not exercise this case. The #(...)* sections
// will each need to be repeated according to the same loop bounds.
// 可选地，允许有多个单独的#(...)*部分，尽管测试不执行此情况。#(...)*部分都需要根据相同的循环边界重复。

use seq::seq;

seq!(N in 0..16 {
    #[derive(Copy, Clone, PartialEq, Debug)]
    enum Interrupt {
        #(
            Irq~N,
        )*
    }
});

fn main() {
    let interrupt = Interrupt::Irq8;

    assert_eq!(interrupt as u8, 8);
    assert_eq!(interrupt, Interrupt::Irq8);
}
