use crate::bytecode::*;
use crate::vm::*;
use std::fmt::{self, Display};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub enum PyObject {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    None,
    Function(Rc<PyFunction>),
    NativeFunction(Rc<PyNativeFunction>),
    Type(PyType),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyType {
    pub name: String,
}

impl Default for PyType {
    fn default() -> Self {
        Self {
            name: "object".to_string(),
        }
    }
}

impl Display for PyObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PyObject::Int(v) => write!(f, "{v}"),
            PyObject::Float(v) => write!(f, "{v}"),
            PyObject::Bool(v) => write!(f, "{v}"),
            PyObject::Str(v) => write!(f, "{}", v),
            PyObject::None => write!(f, "None"),
            PyObject::Function(func) => write!(f, "<function {}>", func.name),
            PyObject::NativeFunction(func) => write!(f, "<native function {}>", func.name),
            PyObject::Type(t) => write!(f, "<type {}>", t.name),
        }
    }
}

impl fmt::Debug for PyObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PyObject::Int(v) => write!(f, "Int({})", v),
            PyObject::Float(v) => write!(f, "Float({})", v),
            PyObject::Bool(v) => write!(f, "Bool({})", v),
            PyObject::Str(v) => write!(f, "Str({:?})", v),
            PyObject::None => write!(f, "None"),
            PyObject::Function(func) => write!(f, "Function({})", func.name),
            PyObject::NativeFunction(func) => write!(f, "NativeFunction({})", func.name),
            PyObject::Type(t) => write!(f, "Type({})", t.name),
        }
    }
}

impl Default for PyObject {
    fn default() -> Self {
        PyObject::None
    }
}

impl From<i64> for PyObject {
    fn from(v: i64) -> Self {
        PyObject::Int(v)
    }
}

impl From<f64> for PyObject {
    fn from(v: f64) -> Self {
        PyObject::Float(v)
    }
}

impl From<bool> for PyObject {
    fn from(v: bool) -> Self {
        PyObject::Bool(v)
    }
}

impl From<&str> for PyObject {
    fn from(v: &str) -> Self {
        PyObject::Str(v.to_string())
    }
}

#[derive(Clone)]
pub struct PyNativeFunction {
    pub name: String,
    pub arity: usize,
    pub func: Rc<dyn Fn(&[PyObject]) -> Result<PyObject, String>>,
}

impl Default for PyNativeFunction {
    fn default() -> Self {
        Self {
            name: String::new(),
            arity: 0,
            func: Rc::new(|_| Ok(PyObject::None)),
        }
    }
}

impl PartialEq for PyNativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}

#[derive(Clone, PartialEq)]
pub struct PyFunction {
    pub name: String,
    pub arity: usize,
    pub code: CodeObject,
    pub globals: Env,
}

impl Default for PyFunction {
    fn default() -> Self {
        Self {
            name: String::new(),
            arity: 0,
            code: CodeObject::default(),
            globals: Env::default(),
        }
    }
}
