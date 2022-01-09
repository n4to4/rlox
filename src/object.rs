#[derive(Debug)]
pub enum Object {
    String(String),
}

impl Object {
    pub fn values_equal(a: &Object, b: &Object) -> bool {
        match (a, b) {
            (Self::String(a), Self::String(b)) => a == b,
        }
    }
}
