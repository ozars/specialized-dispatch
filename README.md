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
        // Defaut implementation.
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

```rust,ignore
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
        default fn <T: Display>(v: T) => {
            format!("default value: {}", v)
        },
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

## Passing Extra Arguments

Extra arguments can be passed to specializations. Argument types need to
declared explicitly (i.e. they won't be captured automatically as it happens
with closures).

```rust
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
    println!("Done!");
}
```

Specialization still happens based on the first argument only.

As with previous examples, the example above is [included][pass_args] in the
repository as well. It can be run with `cargo run --example pass_args` or
inspected with `cargo-expand`.

[pass_args]: examples/pass_args.rs

## Advanced Serdelike Example

Let's say you are implementing a deserializer. There might be certain types
that work well with your own deserializer, while they have a default
implementation for generic deserializers (or even `unimplemented!` by default).

To simplify the example, we will create a watered-down version of relevant
`serde` traits.

```rust
#![feature(min_specialization)]

use specialized_dispatch::specialized_dispatch;

/// A simplified version of `serde::de::Deserializer`.
trait Deserializer<'de> {
    type Error;

    // Some generic deserializer functions...
}

/// A simplified version of `serde::de::Deserialize`.
trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

/// The node type we want to deserialize.
#[derive(Debug)]
struct MyAwesomeNode;

/// Our custom deserializer.
struct MyAwesomeDeserializer;

impl MyAwesomeDeserializer {
    fn my_awesome_function(&mut self) -> MyAwesomeNode {
        MyAwesomeNode
    }
}

impl Deserializer<'_> for MyAwesomeDeserializer {
    type Error = ();
    // Implement the generic deserializer functions...
}

impl<'de> Deserialize<'de> for MyAwesomeNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(specialized_dispatch! {
            D -> MyAwesomeNode,
            // TODO(ozars): This causes rustc ICE.
            // default fn <'de, T: Deserializer<'de>>(_deserializer: T) => {
            default fn <T>(_deserializer: T) => {
                unimplemented!()
            },
            fn (mut deserializer: MyAwesomeDeserializer) => {
                // We can call a method from the concrete implementation here!
                deserializer.my_awesome_function()
            },
            deserializer
        })
    }
}

fn main() {
    println!("{:?}", MyAwesomeNode::deserialize(MyAwesomeDeserializer));
}
```

The example above is [included][serdelike_example] in the repository. It can be
run with `cargo run --example serdelike_example` or inspected with
`cargo-expand`.

[serdelike_example]: examples/serdelike_example.rs

## Limitations

### Requires nightly

This is due to relying on `min_specialization` feature.

### Only concrete types are supported for specialization

Specialization can be used only with concrete types (e.g. subtraits cannot be
used for specialization). This is an existing limitation inherited from the
current implementation of `min_specialization` feature.

### Variables aren't captured automatically

The macro expands its arms to some method implementations. As such, it cannot
refer to other variables in the scope where it's called from.

However, extra arguments can be passed when they are explicitly declared in the
macro. Please refer to [Passing Extra Arguments](#passing-extra-arguments)
section.

### Not working well with lifetimes

I tried implementing lifetime support in various places, but I hit some
compiler errors and in some cases Internal Compiler Errors (ICE). See TODO in
[Advanced Serdelike Example](#advanced-serdelike-example).

This is very likely due to underlying `min_specialization` implementation not
being very mature yet, though it's quite possible I botched something somewhere
(Please file an issue if you figure out which :P).

## Also see

- [sagebind/castaway](https://github.com/sagebind/castaway): Safe, zero-cost
  downcasting for limited compile-time specialization. This is an awesome
  library I've just stumbled upon, which seems to be doing what is done here in
  a more robust and stable way.
