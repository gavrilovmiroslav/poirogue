use caves::Cave;
use serde::{de, Serialize};

pub trait InternalJsonStorage {
    fn get_json<T>(&self, name: &str) -> Option<T>
        where for <'a> T: de::Deserialize<'a>;

    fn set_json<T>(&self, name: &str, value: &T)
        where T: ?Sized + Serialize;
}


impl InternalJsonStorage for Box<dyn Cave + 'static> {
    fn get_json<T>(&self, name: &str) -> Option<T>
        where for <'a> T: de::Deserialize<'a> {
        if let Ok(binary_data) = self.get(name) {
            if let Ok(string_data) = std::str::from_utf8(&binary_data) {
                let json_as_struct = serde_json::from_str::<T>(string_data).unwrap();
                return Some(json_as_struct)
            }
        }

        return None;
    }

    fn set_json<T>(&self, name: &str, value: &T)
        where T: ?Sized + Serialize {

        if let Ok(json) = serde_json::to_string_pretty(value) {
            self.set(name, &json.as_bytes().to_vec()).unwrap();
        }
    }
}