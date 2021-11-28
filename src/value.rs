use std::ops::Index;

#[derive(Debug, Clone, Copy)]
pub struct Value(pub f64);

#[derive(Debug, Clone)]
pub struct ValueArray {
    pub values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        ValueArray { values: Vec::new() }
    }

    pub(crate) fn write_value_array(&mut self, value: Value) {
        self.values.push(value);
    }

    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Index<u8> for ValueArray {
    type Output = Value;

    fn index(&self, offset: u8) -> &Self::Output {
        &self.values[offset as usize]
    }
}
