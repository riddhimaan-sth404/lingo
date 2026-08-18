use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<HashMap<String, Value>>>),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Map(m) => !m.borrow().is_empty(),
        }
    }

    pub fn get_field(&self, key: &str) -> Value {
        match self {
            Value::Map(m) => m.borrow().get(key).cloned().unwrap_or(Value::Nil),
            _ => panic!("Type error: Cannot access field '{}' on {:?}", key, self),
        }
    }

    pub fn set_field(&self, key: impl Into<String>, val: Value) {
        match self {
            Value::Map(m) => {
                m.borrow_mut().insert(key.into(), val);
            }
            _ => panic!("Type error: Cannot set field on {:?}", self),
        }
    }

    pub fn get_index(&self, index: &Value) -> Value {
        match (self, index) {
            (Value::List(l), Value::Int(i)) => {
                let idx = *i as usize;
                l.borrow().get(idx).cloned().unwrap_or(Value::Nil)
            }
            (Value::Map(m), Value::Str(s)) => {
                m.borrow().get(s).cloned().unwrap_or(Value::Nil)
            }
            _ => panic!("Type error: Cannot index {:?} with {:?}", self, index),
        }
    }

    pub fn set_index(&self, index: &Value, val: Value) {
        match (self, index) {
            (Value::List(l), Value::Int(i)) => {
                let idx = *i as usize;
                let mut list = l.borrow_mut();
                if idx >= list.len() {
                    list.resize(idx + 1, Value::Nil);
                }
                list[idx] = val;
            }
            (Value::Map(m), Value::Str(s)) => {
                m.borrow_mut().insert(s.clone(), val);
            }
            _ => panic!("Type error: Cannot set index {:?} with {:?}", self, index),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Str(s) => write!(f, "{}", s),
            Value::List(l) => {
                let items = l.borrow();
                write!(f, "[")?;
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                let items = m.borrow();
                write!(f, "{{")?;
                for (idx, (k, v)) in items.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::List(a), Value::List(b)) => *a.borrow() == *b.borrow(),
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 + b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a + b as f64),
            (Value::Str(a), Value::Str(b)) => Value::Str(format!("{}{}", a, b)),
            (Value::Str(a), b) => Value::Str(format!("{}{}", a, b)),
            (a, Value::Str(b)) => Value::Str(format!("{}{}", a, b)),
            (a, b) => panic!("Type error: Cannot add {:?} and {:?}", a, b),
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 - b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a - b as f64),
            (a, b) => panic!("Type error: Cannot subtract {:?} and {:?}", a, b),
        }
    }
}

impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 * b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a * b as f64),
            (Value::Str(s), Value::Int(n)) => Value::Str(s.repeat(n.max(0) as usize)),
            (a, b) => panic!("Type error: Cannot multiply {:?} and {:?}", a, b),
        }
    }
}

impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Float(a as f64 / b as f64),
            (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 / b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a / b as f64),
            (a, b) => panic!("Type error: Cannot divide {:?} and {:?}", a, b),
        }
    }
}

impl Rem for Value {
    type Output = Value;
    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
            (a, b) => panic!("Type error: Cannot modulo {:?} and {:?}", a, b),
        }
    }
}

// Builtin helper functions used by generated Rust code
pub fn lingo_print(val: &impl fmt::Display) {
    print!("{}", val);
}

pub fn lingo_println(val: &impl fmt::Display) {
    println!("{}", val);
}
