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
    pub fn as_obj(&self) -> Option<&Object> {
        match *self {
            Value::Obj(ref obj) => Some(obj.as_ref()),
            _ => None,
        }
    }

    pub fn new_string(s: String) -> Value {
        Value::Obj(Rc::new(Object::String(s)))
    }

    pub fn string(&self) -> Option<String> {
        match &self {
            Value::Obj(obj) if obj.is_string() => Some(obj.as_str().to_owned()),
            _ => None,
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
