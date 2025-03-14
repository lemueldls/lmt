use lmt_parser::Literal;

pub fn add(left: &Literal, right: &Literal) -> Result<Literal, Box<str>> {
    match (left, right) {
        (Literal::Integer(left), Literal::Integer(right)) => Ok(Literal::Integer(left + right)),
        (Literal::Decimal(left), Literal::Decimal(right)) => Ok(Literal::Decimal(left + right)),
        _ => Err(format!("cannot add {left} and {right}").into_boxed_str()),
    }
}

pub fn divide(left: &Literal, right: &Literal) -> Result<Literal, Box<str>> {
    match (left, right) {
        (Literal::Integer(left), Literal::Integer(right)) => Ok(Literal::Integer(left / right)),
        (Literal::Decimal(left), Literal::Decimal(right)) => Ok(Literal::Decimal(left / right)),
        _ => Err(format!("cannot divide {left} and {right}").into_boxed_str()),
    }
}

pub fn equal(left: &Literal, right: &Literal) -> Result<bool, Box<str>> {
    match (left, right) {
        (Literal::Integer(left), Literal::Integer(right)) => Ok(left == right),
        _ => Err(format!("cannot compare {left} and {right}").into_boxed_str()),
    }
}

pub fn less_than(left: &Literal, right: &Literal) -> Result<bool, Box<str>> {
    match (left, right) {
        (Literal::Integer(left), Literal::Integer(right)) => Ok(left < right),
        (Literal::Decimal(left), Literal::Decimal(right)) => Ok(left < right),
        _ => Err(format!("cannot compare {left} and {right}").into_boxed_str()),
    }
}

pub fn greater_than(left: &Literal, right: &Literal) -> Result<bool, Box<str>> {
    match (left, right) {
        (Literal::Integer(left), Literal::Integer(right)) => Ok(left > right),
        _ => Err(format!("cannot compare {left} and {right}").into_boxed_str()),
    }
}

pub fn max<'a>(left: &'a Literal, right: &'a Literal) -> Result<&'a Literal, Box<str>> {
    match (left, right) {
        (Literal::Integer(left_integer), Literal::Integer(right_integer)) => {
            Ok(if left_integer > right_integer {
                left
            } else {
                right
            })
        }
        _ => Err(format!("cannot compare {left} and {right}").into_boxed_str()),
    }
}

pub fn min<'a>(left: &'a Literal, right: &'a Literal) -> Result<&'a Literal, Box<str>> {
    match (left, right) {
        (Literal::Integer(left_integer), Literal::Integer(right_integer)) => {
            Ok(if left_integer < right_integer {
                left
            } else {
                right
            })
        }
        _ => Err(format!("cannot compare {left} and {right}").into_boxed_str()),
    }
}
