use crate::{PyNativeFunction, PyObject};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::rc::Rc;

pub fn io_module() -> HashMap<String, PyObject> {
    let mut m = HashMap::new();

    m.insert(
        "print".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "print".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                println!("{}", args[0]);
                Ok(PyObject::None)
            }),
        })),
    );

    m.insert(
        "input".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "input".to_string(),
            arity: 0,
            func: Rc::new(|_| {
                let mut buf = String::new();
                io::stdin().read_line(&mut buf).unwrap();
                Ok(PyObject::Str(buf.trim_end().to_string()))
            }),
        })),
    );

    m.insert(
        "read".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "read".to_string(),
            arity: 0,
            func: Rc::new(|_| {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf).unwrap();
                Ok(PyObject::Str(buf))
            }),
        })),
    );

    m.insert(
        "write".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "write".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Str(s) = &args[0] {
                    io::stdout().write_all(s.as_bytes()).unwrap();
                    io::stdout().flush().unwrap();
                    Ok(PyObject::None)
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m
}
