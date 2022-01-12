use super::object::Object;
use std::ops::Index;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
    Obj(Rc<Object>),
}

impl Value {
    pub fn string(&self) -> String {
        match &self {
            Value::Obj(obj) => match obj.as_ref() {
                Object::String(s) => s.to_owned(),
            },
            _ => todo!(),
        }
    }
}

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
        match self {
            Self::Number(number) => write!(f, "{}", number),
            Self::Boolean(bool) => write!(f, "{}", bool),
            Self::Nil => write!(f, "nil"),
            Self::Obj(obj) => match obj.as_ref() {
                Object::String(s) => write!(f, "{}", s),
            },
        }
    }
}

impl Index<u8> for ValueArray {
    type Output = Value;

    fn index(&self, offset: u8) -> &Self::Output {
        &self.values[offset as usize]
    }
}
