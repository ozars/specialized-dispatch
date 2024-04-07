# specialized-dispatch &emsp; [![Crates.io Version]][Crates.io Link]

[Crates.io Version]: https://img.shields.io/crates/v/specialized-dispatch
[Crates.io Link]: https://crates.io/crates/specialized-dispatch

This crate provides a procedural macro, `specialized_dispatch`, a convenient
way to implement different behaviors based on type of an expression.

This works by creating different specializations in the callsite by making use
of [`min_specialization`] nightly feature under the hood.

As such, the caller needs to enable this nightly feature for the library from
which this macro is called.

[`min_specialization`]: https://doc.rust-lang.org/beta/unstable-book/language-features/min-specialization.html

## Simple Example

```rust
#![feature(min_specialization)]

use specialized_dispatch::specialized_dispatch;

fn example<Arg>(arg: Arg) -> String {
    specialized_dispatch!(
        // The argument to the dispatched function. This can be an arbitrary expression.
        arg,
        // Type of the argument -> return type.
        Arg -> String,
        // Defaut implementation. At least one default value is required.
        // Referring to values other than the argument is not supported.
        fn <T>(_: T) => format!("default value"),
        // Specialization for concrete type u8.
        fn (v: u8) => format!("u8: {}", v),
        // Specialization for concrete type u16.
        fn (v: u16) => format!("u16: {}", v),
    )
}

assert_eq!(example(1.0), "default value");
assert_eq!(example(5u8), "u8: 5");
assert_eq!(example(10u16), "u16: 10");
```

`example` function roughly expands to below code. Note that exact expansion is internal
implementation detail. This example is provided to demonstrate how it works under the
hood.

```rust
fn example<T>(t: T) -> String {
    trait SpecializedDispatchCall<T> {
        fn dispatch(t: T) -> String;
    }

    impl<T> SpecializedDispatchCall<T> for T {
        default fn dispatch(_: T) -> String {
            format!("default value")
        }
    }

    impl SpecializedDispatchCall<u8> for u8 {
        fn dispatch(v: u8) -> String {
            format!("u8: {}", v)
        }
    }

    impl SpecializedDispatchCall<u8> for u16 {
        fn dispatch(v: u16) -> String {
            format!("u16: {}", v)
        }
    }

    <Arg as SpecializedDispatchCall<Arg>>::dispatch(arg)
}
```

