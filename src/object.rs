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

    pub fn is_string(&self) -> bool {
        matches!(*self, Object::String(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Object::String(s) => s.as_str(),
        }
    }
}
