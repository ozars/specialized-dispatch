#![feature(min_specialization)]

use specialized_dispatch::specialized_dispatch;

fn example<E>(expr: E) -> String {
    specialized_dispatch!(
        // Type of the expression -> return type.
        E -> String,
        // Defaut implementation. At least one default value is required.
        // Referring to values other than the provided argument is not
        // supported.
        default fn <T>(_: T) => format!("default value"),
        // Specialization for concrete type u8.
        fn (v: u8) => format!("u8: {}", v),
        // Specialization for concrete type u16.
        fn (v: u16) => format!("u16: {}", v),
        // The expression for passing to above specializations.
        expr,
    )
}

fn main() {
    assert_eq!(example(1.0), "default value");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
    println!("Done!");
}
