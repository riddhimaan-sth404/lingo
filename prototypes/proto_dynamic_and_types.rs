//! Prototype A & B: Dynamic Values & Optional Type Constraints
//! Validates tagged runtime values and dynamic-to-static constraint transitions.

#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl DynamicValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            DynamicValue::Nil => false,
            DynamicValue::Bool(b) => *b,
            DynamicValue::Int(i) => *i != 0,
            DynamicValue::Float(f) => *f != 0.0,
            DynamicValue::Str(s) => !s.is_empty(),
        }
    }

    pub fn to_int(&self) -> Result<i64, String> {
        match self {
            DynamicValue::Int(i) => Ok(*i),
            DynamicValue::Float(f) => Ok(*f as i64),
            other => Err(format!("Cannot convert {:?} to integer", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_reassignment_and_truthiness() {
        let mut x = DynamicValue::Int(0);
        assert!(!x.is_truthy());
        x = DynamicValue::Str("hello".into());
        assert!(x.is_truthy());
    }

    #[test]
    fn test_optional_type_constraints() {
        let dynamic_val = DynamicValue::Int(42);
        let constrained_i64: i64 = dynamic_val.to_int().expect("Valid constraint conversion");
        assert_eq!(constrained_i64, 42);
    }
}
