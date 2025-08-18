use crate::object::*;
use std::collections::HashMap;

#[derive(Clone, Default, PartialEq)]
pub struct Env {
    pub locals: HashMap<String, PyObject>,
    pub globals: HashMap<String, PyObject>,
    pub builtins: HashMap<String, PyObject>,
}
