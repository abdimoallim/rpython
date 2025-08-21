use crate::{PyNativeFunction, PyObject};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;

pub fn sys_module() -> HashMap<String, PyObject> {
    let argv = env::args().map(PyObject::Str).collect::<Vec<_>>();
    let path = env::var("PYTHONPATH")
        .unwrap_or_default()
        .split(':')
        .map(|s| PyObject::Str(s.to_string()))
        .collect::<Vec<_>>();

    let mut m = HashMap::new();

    m.insert(
        "argv".to_string(),
        PyObject::List(Rc::new(RefCell::new(argv))),
    );
    m.insert(
        "executable".to_string(),
        PyObject::Str(env::current_exe().unwrap().to_string_lossy().to_string()),
    );
    m.insert(
        "version".to_string(),
        PyObject::Str(env!("CARGO_PKG_VERSION").to_string()),
    );
    m.insert(
        "platform".to_string(),
        PyObject::Str(std::env::consts::OS.to_string()),
    );
    m.insert(
        "byteorder".to_string(),
        PyObject::Str(
            if cfg!(target_endian = "little") {
                "little"
            } else {
                "big"
            }
            .to_string(),
        ),
    );
    m.insert("maxsize".to_string(), PyObject::Int(std::usize::MAX as i64));
    m.insert(
        "modules".to_string(),
        PyObject::Dict(Rc::new(RefCell::new(IndexMap::new()))),
    );
    m.insert(
        "path".to_string(),
        PyObject::List(Rc::new(RefCell::new(path))),
    );
    m.insert(
        "stdin".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "stdin".to_string(),
            arity: 0,
            func: Rc::new(|_| Ok(PyObject::None)),
        })),
    );
    m.insert(
        "stdout".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "stdout".to_string(),
            arity: 0,
            func: Rc::new(|_| Ok(PyObject::None)),
        })),
    );
    m.insert(
        "stderr".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "stderr".to_string(),
            arity: 0,
            func: Rc::new(|_| Ok(PyObject::None)),
        })),
    );
    m.insert(
        "exit".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "exit".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Int(code) = args[0] {
                    std::process::exit(code as i32);
                }
                Err("bad args".to_string())
            }),
        })),
    );
    m.insert(
        "getsizeof".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "getsizeof".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                let size = match &args[0] {
                    PyObject::Int(_) => std::mem::size_of::<i64>(),
                    PyObject::Float(_) => std::mem::size_of::<f64>(),
                    PyObject::Str(s) => std::mem::size_of::<String>() + s.len(),
                    PyObject::List(l) => {
                        std::mem::size_of::<Rc<RefCell<Vec<PyObject>>>>()
                            + l.borrow().len() * std::mem::size_of::<PyObject>()
                    }
                    PyObject::Dict(d) => {
                        std::mem::size_of::<Rc<RefCell<HashMap<String, PyObject>>>>()
                            + d.borrow().len() * std::mem::size_of::<(String, PyObject)>()
                    }
                    PyObject::NativeFunction(_) => std::mem::size_of::<Rc<PyNativeFunction>>(),
                    PyObject::NativeModule(_) => std::mem::size_of::<Rc<crate::PyNativeModule>>(),
                    PyObject::None => 0,
                    _ => 0,
                };
                Ok(PyObject::Int(size as i64))
            }),
        })),
    );
    m.insert("recursionlimit".to_string(), PyObject::Int(1 << 10));
    m.insert(
        "version_info".to_string(),
        PyObject::Tuple(vec![
            PyObject::Int(3),
            PyObject::Int(11),
            PyObject::Int(6),
            PyObject::Str("final".to_string()),
            PyObject::Int(0),
        ]),
    );

    m
}
