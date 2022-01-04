use pickledb::PickleDb;

pub trait StoreHelpers {
    fn lregen(&mut self, name: &str);
}

impl StoreHelpers for PickleDb {
    fn lregen(&mut self, name: &str) {
        self.lrem_list(name).unwrap();
        self.lcreate(name).unwrap();
    }
}