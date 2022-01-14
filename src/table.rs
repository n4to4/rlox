use super::value::Value;

pub struct Table {
    count: u32,
    capacity: u32,
    entries: Option<Box<Entry>>,
}

pub struct Entry {
    key: String,
    value: Value,
}

impl Table {
    pub fn new() -> Self {
        Table {
            count: 0,
            capacity: 0,
            entries: None,
        }
    }
}
