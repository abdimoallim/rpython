use std::collections::HashSet;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::object::{PyNativeFunction, PyObject, PyType};

pub fn apply(builtins: &mut HashMap<String, PyObject>) {
    builtins.insert(
        "set".to_string(), /*@todo: class*/
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "set".to_string(),
            arity: 0,
            func: Rc::new(|_| Ok(PyObject::Set(Rc::new(RefCell::new(HashSet::new()))))),
        })),
    );

    builtins.insert(
        "print".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "print".to_string(),
            arity: usize::MAX,
            func: Rc::new(|args| {
                let mut fst = true;

                for a in args {
                    if !fst {
                        print!(" ");
                    }

                    fst = false;

                    print!("{}", a);
                }

                println!();

                Ok(PyObject::None)
            }),
        })),
    );

    builtins.insert(
        "range".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "range".to_string(),
            arity: usize::MAX,
            func: Rc::new(|args| {
                let (start, stop, step) = match args.len() {
                    1 => {
                        if let PyObject::Int(stop) = &args[0] {
                            (0, *stop, 1)
                        } else {
                            return Err(
                                "TypeError: range() argument must be an integer".to_string()
                            );
                        }
                    }
                    2 => {
                        if let (PyObject::Int(start), PyObject::Int(stop)) = (&args[0], &args[1]) {
                            (*start, *stop, 1)
                        } else {
                            return Err("TypeError: range() arguments must be integers".to_string());
                        }
                    }
                    3 => {
                        if let (PyObject::Int(start), PyObject::Int(stop), PyObject::Int(step)) =
                            (&args[0], &args[1], &args[2])
                        {
                            if *step == 0 {
                                return Err(
                                    "ValueError: range() arg 3 must not be zero".to_string()
                                );
                            }
                            (*start, *stop, *step)
                        } else {
                            return Err("TypeError: range() arguments must be integers".to_string());
                        }
                    }
                    _ => return Err("TypeError: range expected 1 to 3 arguments".to_string()),
                };

                let mut items = Vec::new();
                if step > 0 {
                    let mut i = start;
                    while i < stop {
                        items.push(PyObject::Int(i));
                        i += step;
                    }
                } else {
                    let mut i = start;
                    while i > stop {
                        items.push(PyObject::Int(i));
                        i += step;
                    }
                }

                Ok(PyObject::List(Rc::new(RefCell::new(items))))
            }),
        })),
    );

    builtins.insert(
        "type".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "type".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                let t = match &args[0] {
                    PyObject::Int(_) => PyType {
                        name: "int".to_string(),
                    },
                    PyObject::Float(_) => PyType {
                        name: "float".to_string(),
                    },
                    PyObject::Bool(_) => PyType {
                        name: "bool".to_string(),
                    },
                    PyObject::Str(_) => PyType {
                        name: "str".to_string(),
                    },
                    PyObject::List(_) => PyType {
                        name: "list".to_string(),
                    },
                    PyObject::Dict(_) => PyType {
                        name: "dict".to_string(),
                    },
                    PyObject::Tuple(_) => PyType {
                        name: "tuple".to_string(),
                    },
                    PyObject::Set(_) => PyType {
                        name: "set".to_string(),
                    },
                    PyObject::None => PyType {
                        name: "NoneType".to_string(),
                    },
                    PyObject::Function(_) => PyType {
                        name: "function".to_string(),
                    },
                    PyObject::NativeFunction(_) => PyType {
                        name: "native_function".to_string(),
                    },
                    PyObject::NativeModule(_) => PyType {
                        name: "module".to_string(),
                    },
                    PyObject::NativeClass(_) => PyType {
                        name: "type".to_string(),
                    },
                    PyObject::Type(_) => PyType {
                        name: "type".to_string(),
                    },
                    PyObject::Class(_) => PyType {
                        name: "type".to_string(),
                    },
                    PyObject::Instance(inst) => PyType {
                        name: inst.borrow().class.name.clone(),
                    },
                    PyObject::Module(_) => PyType {
                        name: "module".to_string(),
                    },
                };

                Ok(PyObject::Type(t))
            }),
        })),
    );
}
