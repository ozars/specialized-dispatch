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
                deserializer.my_awesome_function()
            },
            deserializer
        })
    }
}

fn main() {
    println!("{:?}", MyAwesomeNode::deserialize(MyAwesomeDeserializer));
}
