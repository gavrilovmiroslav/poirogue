use std::collections::HashSet;

pub type Item = String;

pub struct HasInventory(pub HashSet<Item>);
