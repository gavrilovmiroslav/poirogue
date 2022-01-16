use std::fs::{File, remove_file};
use std::path::Path;
use caves::Cave;
use caves::res::Res;
use filearco::v1::FileArco;
use lru::{DefaultHasher, LruCache};


pub struct ReadonlyArchiveCave {
    archive: FileArco,
}

impl ReadonlyArchiveCave {
    pub fn make_from(dir: &str, output: &str) {
        let data_path = Path::new(dir);
        let output_path = Path::new(output);
        if Path::exists(output_path) {
            remove_file(output_path).expect("Old binarized file archive removed successfully");
        }

        let output = File::create(output_path).unwrap();
        let file_data = filearco::get_file_data(data_path).ok().unwrap();
        FileArco::make(file_data, output).expect("Binarized file archive created successfully");
    }

    pub fn open(path: String) -> ReadonlyArchiveCave {
        let archive = FileArco::new(path.as_str()).unwrap();
        ReadonlyArchiveCave { archive }
    }
}

impl Cave for ReadonlyArchiveCave {
    fn get(&self, name: &str) -> Res {
        let file = self.archive.get(name).unwrap();
        let data = file.as_slice().to_vec();
        Ok(data)
    }

    fn set(&self, name: &str, data: &[u8]) -> Res {
        panic!("You're attempting to set values in a read-only archive.");
    }

    fn delete(&self, name: &str) -> Res {
        panic!("You're attempting to delete values in a read-only archive.");
    }
}