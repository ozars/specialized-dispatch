#![feature(min_specialization)]

use specialized_dispatch::specialized_dispatch;

#[test]
fn test_example() {
    fn example<E>(expr: E) -> String {
        specialized_dispatch!(
            E -> String,
            default fn <T>(_: T) => format!("default value"),
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
            expr,
        )
    }

    assert_eq!(example(1.0), "default value");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
}

#[test]
fn test_example_different_order() {
    fn example<E>(expr: E) -> String {
        specialized_dispatch!(
            E -> String,
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
            default fn <T>(_: T) => format!("default value"),
            expr,
        )
    }

    assert_eq!(example(1.0), "default value");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
}

#[test]
fn test_multiple_calls_in_same_scope() {
    let s1 = specialized_dispatch!(
        u8 -> &'static str,
        fn (_: u8) => "u8",
        fn (_: u16) => "u16",
        default fn <T>(_: T) => "other",
        0u8,
    );
    let s2 = specialized_dispatch!(
        u16 -> &'static str,
        fn (_: u8) => "u8",
        fn (_: u16) => "u16",
        default fn <T>(_: T) => "other",
        0u16,
    );
    assert_eq!(format!("{}-{}", s1, s2), "u8-u16");
}

#[test]
fn test_bound_traits() {
    use std::fmt::{Debug, Display};

    fn example<E: Display + Debug>(expr: E) -> String {
        specialized_dispatch!(
            E -> String,
            default fn <T: Display + Debug>(v: T) => format!("default value: {}", v),
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
            expr,
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

    fn example<E: Display + GenericTrait<()>>(expr: E) -> String {
        specialized_dispatch!(
            E -> String,
            default fn <T: Display + GenericTrait<()>>(v: T) => format!("default value: {}", v),
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
            expr,
        )
    }

    assert_eq!(example(1.5), "default value: 1.5");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
}

#[test]
fn test_extra_args() {
    use std::fmt::Display;
    fn example<T: Display>(expr: T, arg: u8) -> String {
        specialized_dispatch!(
            T -> String,
            default fn <T: Display>(v: T, arg: u8) => format!("default value: {}, arg: {}", v, arg),
            fn (v: u8, arg: u8) => format!("u8: {}, arg: {}", v, arg),
            fn (v: u16, arg: u8) => format!("u16: {}, arg: {}", v, arg),
            expr, arg,
        )
    }

    assert_eq!(example(1.5, 123u8), "default value: 1.5, arg: 123");
    assert_eq!(example(5u8, 12u8), "u8: 5, arg: 12");
    assert_eq!(example(10u16, 1u8), "u16: 10, arg: 1");
}

#[test]
fn test_extra_args_with_str_arg() {
    use std::fmt::Display;
    fn example<T: Display>(expr: T, arg: &str) -> String {
        specialized_dispatch!(
            T -> String,
            default fn <T: Display>(v: T, arg: &str) => format!("default value: {}, arg: {}", v, arg),
            fn (v: u8, arg: &str) => format!("u8: {}, arg: {}", v, arg),
            fn (v: u16, arg: &str) => format!("u16: {}, arg: {}", v, arg),
            expr, arg,
        )
    }

    assert_eq!(example(1.5, "ben bir"), "default value: 1.5, arg: ben bir");
    assert_eq!(example(5u8, "ceviz"), "u8: 5, arg: ceviz");
    assert_eq!(example(10u16, "agaciyim"), "u16: 10, arg: agaciyim");
}
