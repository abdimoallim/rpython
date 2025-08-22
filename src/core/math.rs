use crate::{PyNativeFunction, PyObject};
use std::collections::HashMap;
use std::rc::Rc;

pub fn math_module() -> HashMap<String, PyObject> {
    let mut m = HashMap::new();

    m.insert("pi".to_string(), PyObject::Float(std::f64::consts::PI));
    m.insert("e".to_string(), PyObject::Float(std::f64::consts::E));
    m.insert("tau".to_string(), PyObject::Float(std::f64::consts::TAU));
    m.insert("inf".to_string(), PyObject::Float(f64::INFINITY));
    m.insert("nan".to_string(), PyObject::Float(f64::NAN));

    m.insert(
        "sin".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "sin".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.sin()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "cos".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "cos".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.cos()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "tan".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "tan".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.tan()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "asin".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "asin".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.asin()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "acos".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "acos".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.acos()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "atan".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "atan".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.atan()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "sqrt".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "sqrt".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.sqrt()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "log".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "log".to_string(),
            arity: 2,
            func: Rc::new(|args| {
                let x = match args[0] {
                    PyObject::Float(v) => v,
                    _ => return Err("bad args".to_string()),
                };
                let base = match args[1] {
                    PyObject::Float(v) => v,
                    _ => return Err("bad args".to_string()),
                };
                Ok(PyObject::Float(x.log(base)))
            }),
        })),
    );

    m.insert(
        "log2".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "log2".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.log2()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "log10".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "log10".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.log10()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "exp".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "exp".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.exp()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "fabs".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "fabs".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Float(x.abs()))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "floor".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "floor".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Int(x.floor() as i64))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "ceil".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "ceil".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    Ok(PyObject::Int(x.ceil() as i64))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m.insert(
        "round".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "round".to_string(),
            arity: 2,
            func: Rc::new(|args| {
                if let PyObject::Float(x) = args[0] {
                    let ndigits = match args[1] {
                        PyObject::Int(v) => v,
                        _ => return Err("bad args".to_string()),
                    };
                    let factor = 10f64.powi(ndigits as i32);
                    Ok(PyObject::Float((x * factor).round() / factor))
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m
}
