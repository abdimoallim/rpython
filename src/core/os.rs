use std::collections::HashMap;
use std::env as sys_env;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Env, PyNativeFunction, PyObject};

pub fn os_module() -> HashMap<String, PyObject> {
    let mut m = HashMap::new();

    m.insert(
        "getcwd".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "getcwd".to_string(),
            arity: 0,
            func: Rc::new(|_| {
                Ok(PyObject::Str(
                    sys_env::current_dir()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                ))
            }),
        })),
    );

    m.insert(
        "time".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "time".to_string(),
            arity: 0,
            func: Rc::new(|_| {
                Ok(PyObject::Float(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                ))
            }),
        })),
    );

    m.insert(
        "getenv".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "getenv".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Str(ref key) = args[0] {
                    Ok(PyObject::Str(sys_env::var(key).unwrap_or_default()))
                } else {
                    Err("bad args".to_string())
                }
            }),
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

    m
}
