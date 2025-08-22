use crate::{PyNativeFunction, PyObject};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn time_module() -> HashMap<String, PyObject> {
    let mut m = HashMap::new();

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
        "sleep".to_string(),
        PyObject::NativeFunction(Rc::new(PyNativeFunction {
            name: "sleep".to_string(),
            arity: 1,
            func: Rc::new(|args| {
                if let PyObject::Float(sec) = args[0] {
                    std::thread::sleep(Duration::from_secs_f64(sec));
                    Ok(PyObject::None)
                } else if let PyObject::Int(sec) = args[0] {
                    std::thread::sleep(Duration::from_secs(sec as u64));
                    Ok(PyObject::None)
                } else {
                    Err("bad args".to_string())
                }
            }),
        })),
    );

    m
}
