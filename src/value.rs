use std::ops::Index;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Value(pub(crate) f64);

#[derive(Debug)]
pub(crate) struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub(crate) fn new() -> Self {
        ValueArray { values: Vec::new() }
    }

    pub(crate) fn write_value_array(&mut self, value: Value) {
        self.values.push(value);
    }

    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }
}

impl Index<u8> for ValueArray {
    type Output = Value;

    fn index(&self, offset: u8) -> &Self::Output {
        &self.values[offset as usize]
    }
}
