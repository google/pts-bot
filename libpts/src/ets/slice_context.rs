use evalexpr::error::{EvalexprError, EvalexprResult};
use evalexpr::{Context, Value};

const TRUE: Value = Value::Boolean(true);
const FALSE: Value = Value::Boolean(false);

pub struct SliceContext<'a>(pub &'a [(&'a str, bool)]);

impl<'a> Context for SliceContext<'a> {
    fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.0
            .iter()
            .find(|(name, _)| *name == identifier)
            .map(|(_, value)| if *value { &TRUE } else { &FALSE })
    }

    fn call_function(&self, identifier: &str, _argument: &Value) -> EvalexprResult<Value> {
        Err(EvalexprError::FunctionIdentifierNotFound(
            identifier.to_string(),
        ))
    }
}
