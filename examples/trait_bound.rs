#![feature(min_specialization)]

use std::fmt::Display;

use specialized_dispatch::specialized_dispatch;

// The argument type must also bind to the same trait.
fn example<Arg: Display>(arg: Arg) -> String {
    specialized_dispatch!(
        arg,
        Arg -> String,
        // Notice the trait bound.
        fn <T: Display>(v: T) => format!("default value: {}", v),
        // Note that specializations also need to satisfy the same bound.
        fn (v: u8) => format!("u8: {}", v),
        fn (v: u16) => format!("u16: {}", v),
    )
}

fn main() {
    assert_eq!(example(1.5), "default value: 1.5");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
    println!("Done!");
}
