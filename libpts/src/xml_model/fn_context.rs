// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
