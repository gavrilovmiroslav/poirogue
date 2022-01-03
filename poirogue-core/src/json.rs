use serde::{de, Serialize};

pub trait JsonFields {
    fn get_json<T>(&self, name: &str) -> Option<T>
        where for <'a> T: de::Deserialize<'a>;

    fn set_json<T>(&self, name: &str, value: &T)
        where T: ?Sized + Serialize;
}