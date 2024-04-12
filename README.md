<h1 align="center">specialized-dispatch</h1>

<p align="center">
  <a href="https://github.com/ozars/specialized-dispatch"><img alt="Github Repository" src="https://img.shields.io/badge/ozars%2Fspecialized--dispatch-8da0cb?style=for-the-badge&logo=github&label=github"></a>
  <a href="https://crates.io/crates/specialized-dispatch"><img alt="crates.io Version" src="https://img.shields.io/crates/v/specialized-dispatch?style=for-the-badge&logo=rust"></a>
  <a href="https://docs.rs/specialized-dispatch/latest/specialized_dispatch/"><img alt="docs.rs Documentation" src="https://img.shields.io/docsrs/specialized-dispatch?style=for-the-badge&logo=docs.rs"></a>
  <a href="https://github.com/ozars/specialized-dispatch/actions/workflows/rust.yml"><img alt="Github Actions Build" src="https://img.shields.io/github/actions/workflow/status/ozars/specialized-dispatch/rust.yml?style=for-the-badge&logo=github-actions"></a>
</p>

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
```

`example` function roughly expands to below code. Note that exact expansion is
internal implementation detail. This example is provided to demonstrate how it
works under the hood.

```rust
fn example<E>(expr: E) -> String {
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

    <E as SpecializedDispatchCall<E>>::dispatch(expr)
}
```

The example above is [included][simple_example] in the repository.

It can be run with `cargo run --example simple_example`.

Expanded code can be inspected using [`cargo-expand`]: `cargo expand --example
simple_example`.

[simple_example]: examples/simple_example.rs
[`cargo-expand`]: https://crates.io/crates/cargo-expand

## Trait Bounds

Trait bounds can be provided for the default case:

```rust
#![feature(min_specialization)]

use std::fmt::Display;

use specialized_dispatch::specialized_dispatch;

// The expression type must also bind to the same trait.
fn example<E: Display>(expr: E) -> String {
    specialized_dispatch!(
        E -> String,
        // Notice the trait bound.
        default fn <T: Display>(v: T) => format!("default value: {}", v),
        // Note that specializations also need to satisfy the same bound.
        fn (v: u8) => format!("u8: {}", v),
        fn (v: u16) => format!("u16: {}", v),
        expr,
    )
}

fn main() {
    assert_eq!(example(1.5), "default value: 1.5");
    assert_eq!(example(5u8), "u8: 5");
    assert_eq!(example(10u16), "u16: 10");
    println!("Done!");
}
```

Likewise, the example above is [included][trait_bound] in the repository.

It can be run with `cargo run --example trait_bound` or inspected with
`cargo-expand`.

[trait_bound]: examples/trait_bound.rs

## Limitations

### Requires nightly

This is due to relying on `min_specialization` feature.

### Only concrete types are supported for specialization

Specialization can be used only with concrete types (e.g. subtraits cannot be
used for specialization). This is an existing limitation inherited from the
current implementation of `min_specialization` feature.

### No variables other than the argument can be referred

The macro expands its arms to some method implementations. As such, it cannot
refer to other variables in the scope where it's called from.

