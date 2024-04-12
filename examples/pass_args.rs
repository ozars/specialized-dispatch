#![feature(min_specialization)]

use std::fmt::Display;

use specialized_dispatch::specialized_dispatch;

fn example<T: Display>(expr: T, arg: &str) -> String {
    specialized_dispatch!(
        T -> String,
        default fn <T: Display>(v: T, arg: &str) => {
            format!("default value: {}, arg: {}", v, arg)
        },
        fn (v: u8, arg: &str) => format!("u8: {}, arg: {}", v, arg),
        fn (v: u16, arg: &str) => format!("u16: {}, arg: {}", v, arg),
        expr, arg,
    )
}

fn main() {
    assert_eq!(example(1.5, "I'm a"), "default value: 1.5, arg: I'm a");
    assert_eq!(example(5u8, "walnut"), "u8: 5, arg: walnut");
    assert_eq!(example(10u16, "tree"), "u16: 10, arg: tree");
}
