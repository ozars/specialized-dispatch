#![feature(min_specialization)]

use specialized_dispatch::specialized_dispatch;

#[test]
fn test_example() {
    fn example<Arg>(arg: Arg) -> String {
        specialized_dispatch!(
            arg,
            Arg -> String,
            fn <T>(_: T) => format!("default value"),
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
        )
    }

    assert_eq!(example(1.0), "default value");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
}

#[test]
fn test_example_different_order() {
    fn example<Arg>(arg: Arg) -> String {
        specialized_dispatch!(
            arg,
            Arg -> String,
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
            fn <T>(_: T) => format!("default value"),
        )
    }

    assert_eq!(example(1.0), "default value");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
}

#[test]
fn test_multiple_calls_in_same_scope() {
    let s1 = specialized_dispatch!(
        0u8,
        u8 -> &'static str,
        fn (_: u8) => "u8",
        fn (_: u16) => "u16",
        fn <T>(_: T) => "other",
    );
    let s2 = specialized_dispatch!(
        0u16,
        u16 -> &'static str,
        fn (_: u8) => "u8",
        fn (_: u16) => "u16",
        fn <T>(_: T) => "other",
    );
    assert_eq!(format!("{}-{}", s1, s2), "u8-u16");
}

#[test]
fn test_bound_traits() {
    use std::fmt::{Debug, Display};

    fn example<Arg: Display + Debug>(arg: Arg) -> String {
        specialized_dispatch!(
            arg,
            Arg -> String,
            fn <T: Display + Debug>(v: T) => format!("default value: {}", v),
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
        )
    }

    assert_eq!(example(1.5), "default value: 1.5");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
}

#[test]
fn test_bound_traits_with_generic() {
    use std::fmt::Display;
    trait GenericTrait<T> {}
    impl GenericTrait<()> for f32 {}
    impl GenericTrait<()> for u8 {}
    impl GenericTrait<()> for u16 {}

    fn example<Arg: Display + GenericTrait<()>>(arg: Arg) -> String {
        specialized_dispatch!(
            arg,
            Arg -> String,
            fn <T: Display + GenericTrait<()>>(v: T) => format!("default value: {}", v),
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
        )
    }

    assert_eq!(example(1.5), "default value: 1.5");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
}
