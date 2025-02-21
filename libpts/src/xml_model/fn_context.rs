use evalexpr::error::{EvalexprError, EvalexprResult};
use evalexpr::{Context, Value};

const TRUE: Value = Value::Boolean(true);
const FALSE: Value = Value::Boolean(false);

pub struct FnContext<'a, T>(pub &'a T);

impl<T> Context for FnContext<'_, T>
where
    T: Fn(&str) -> Option<bool>,
{
    fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.0(identifier).map(|value| if value { &TRUE } else { &FALSE })
    }

    fn call_function(&self, identifier: &str, _argument: &Value) -> EvalexprResult<Value> {
        Err(EvalexprError::FunctionIdentifierNotFound(
            identifier.to_string(),
        ))
    }
}
