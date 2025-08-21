use crate::bytecode::*;
use crate::vm::*;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub enum PyObject {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Rc<RefCell<Vec<PyObject>>>),
    Dict(Rc<RefCell<IndexMap<String, PyObject>>>),
    Tuple(Vec<PyObject>),
    Set(Rc<RefCell<HashSet<PyObject>>>),
    None,
    Function(Rc<PyFunction>),
    NativeFunction(Rc<PyNativeFunction>),
    NativeModule(Rc<PyNativeModule>),
    NativeClass(Rc<PyNativeClass>),
    Type(PyType),
    Class(Rc<PyClass>),
    Instance(Rc<RefCell<PyInstance>>),
    Module(Rc<RefCell<PyModule>>),
}

#[derive(Clone, PartialEq)]
pub struct PyModule {
    pub name: String,
    pub dict: HashMap<String, PyObject>,
}

#[derive(Clone, PartialEq)]
pub struct PyClass {
    pub name: String,
    pub methods: HashMap<String, PyObject>,
    pub bases: Vec<Rc<PyClass>>,
}

#[derive(Clone, PartialEq)]
pub struct PyInstance {
    pub class: Rc<PyClass>,
    pub attrs: HashMap<String, PyObject>,
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
            PyObject::List(l) => {
                let items: Vec<String> = l.borrow().iter().map(|x| format!("{}", x)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            PyObject::Dict(d) => {
                let items: Vec<String> = d
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("'{}': {}", k, v))
                    .collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            PyObject::Tuple(t) => {
                let items: Vec<String> = t.iter().map(|x| format!("{}", x)).collect();
                if t.len() == 1 {
                    write!(f, "({},)", items[0])
                } else {
                    write!(f, "({})", items.join(", "))
                }
            }
            PyObject::Set(s) => {
                let items: Vec<String> = s.borrow().iter().map(|x| format!("{}", x)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            PyObject::None => write!(f, "None"),
            PyObject::Function(func) => write!(f, "<function {}>", func.name),
            PyObject::NativeFunction(func) => write!(f, "<native function {}>", func.name),
            PyObject::NativeModule(m) => write!(f, "<module '{}'>", m.name),
            PyObject::NativeClass(c) => write!(f, "<class '{}'>", c.name),
            PyObject::Type(t) => write!(f, "<type {}>", t.name),
            PyObject::Class(c) => write!(f, "<class '{}'>", c.name),
            PyObject::Instance(i) => write!(f, "<{} object>", i.borrow().class.name),
            PyObject::Module(m) => write!(f, "<module '{}'>", m.borrow().name),
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
            PyObject::List(l) => write!(f, "List({:?})", l.borrow().as_slice()),
            PyObject::Dict(d) => write!(f, "Dict({:?})", d.borrow()),
            PyObject::Tuple(t) => write!(f, "Tuple({:?})", t),
            PyObject::Set(s) => write!(f, "Set({:?})", s.borrow()),
            PyObject::None => write!(f, "None"),
            PyObject::Function(func) => write!(f, "Function({})", func.name),
            PyObject::NativeFunction(func) => write!(f, "NativeFunction({})", func.name),
            PyObject::NativeModule(m) => write!(f, "NativeModule({})", m.name),
            PyObject::NativeClass(c) => write!(f, "NativeClass({})", c.name),
            PyObject::Type(t) => write!(f, "Type({})", t.name),
            PyObject::Class(c) => write!(f, "Class({})", c.name),
            PyObject::Instance(i) => write!(f, "Instance({})", i.borrow().class.name),
            PyObject::Module(m) => write!(f, "Module({})", m.borrow().name),
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

impl std::hash::Hash for PyObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            PyObject::Int(v) => v.hash(state),
            PyObject::Float(v) => v.to_bits().hash(state),
            PyObject::Bool(v) => v.hash(state),
            PyObject::Str(v) => v.hash(state),
            PyObject::None => 0.hash(state),
            _ => panic!("unhashable type"),
        }
    }
}

impl Eq for PyObject {}

#[derive(Clone)]
pub struct PyNativeModule {
    pub name: String,
    pub dict: HashMap<String, PyObject>,
}

impl PartialEq for PyNativeModule {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone)]
pub struct PyNativeClass {
    pub name: String,
    pub methods: HashMap<String, PyObject>,
    pub constructor: Rc<dyn Fn(&[PyObject]) -> Result<PyObject, String>>,
}

impl PartialEq for PyNativeClass {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
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
