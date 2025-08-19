use crate::bytecode::*;
use crate::object::*;
use crate::opcode::*;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Default, PartialEq)]
pub struct Env {
    pub locals: HashMap<String, PyObject>,
    pub globals: HashMap<String, PyObject>,
    pub builtins: HashMap<String, PyObject>,
}

#[derive(Default)]
pub struct Vm {
    pub stack: Vec<PyObject>,
    pub env: Env,
}

impl Vm {
    pub fn with_builtins(mut self) -> Self {
        self.env.builtins.insert(
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

        self.env.builtins.insert(
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
                        PyObject::None => PyType {
                            name: "NoneType".to_string(),
                        },
                        PyObject::Function(_) => PyType {
                            name: "function".to_string(),
                        },
                        PyObject::NativeFunction(_) => PyType {
                            name: "native_function".to_string(),
                        },
                        PyObject::Type(_) => PyType {
                            name: "type".to_string(),
                        },
                    };

                    Ok(PyObject::Type(t))
                }),
            })),
        );

        self
    }

    pub fn register_native<F>(&mut self, name: &str, arity: usize, f: F)
    where
        F: Fn(&[PyObject]) -> Result<PyObject, String> + 'static,
    {
        self.env.globals.insert(
            name.to_string(),
            PyObject::NativeFunction(Rc::new(PyNativeFunction {
                name: name.to_string(),
                arity,
                func: Rc::new(f),
            })),
        );
    }

    pub fn run(&mut self, code: &CodeObject) -> Result<PyObject, String> {
        let mut ip = 0usize;
        let mut frames: Vec<(usize, CodeObject, Env)> = Vec::new();
        let mut cur = code.clone();

        // dbg!(cur.instructions.clone());
        cur.debug_print();

        loop {
            if ip >= cur.instructions.len() {
                return Ok(PyObject::None);
            }

            match cur.instructions[ip] {
                Op::LoadConst(idx) => {
                    self.stack.push(cur.consts[idx].clone());
                    ip += 1;
                }
                Op::LoadName(idx) => {
                    let name = &cur.names[idx];
                    if let Some(v) = self.env.locals.get(name) {
                        self.stack.push(v.clone());
                    } else if let Some(v) = self.env.globals.get(name) {
                        self.stack.push(v.clone());
                    } else if let Some(v) = self.env.builtins.get(name) {
                        self.stack.push(v.clone());
                    } else {
                        return Err(format!("NameError: name '{}' is not defined", name));
                    }

                    ip += 1;
                }
                Op::StoreName(idx) => {
                    let name = cur.names[idx].clone();
                    let v = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.env.locals.insert(name, v);
                    ip += 1;
                }
                Op::LoadGlobal(idx) => {
                    let name = &cur.names[idx];
                    if let Some(v) = self
                        .env
                        .globals
                        .get(name)
                        .cloned()
                        .or_else(|| self.env.builtins.get(name).cloned())
                    {
                        self.stack.push(v);
                        ip += 1;
                    } else {
                        return Err(format!("NameError: global '{}' is not defined", name));
                    }
                }
                Op::StoreGlobal(idx) => {
                    let name = cur.names[idx].clone();
                    let v = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.env.globals.insert(name, v);
                    ip += 1;
                }
                Op::Pop => {
                    self.stack.pop();
                    ip += 1;
                }
                Op::Return => {
                    let ret = self.stack.pop().unwrap_or(PyObject::None);
                    if let Some((rip, parent, saved_env)) = frames.pop() {
                        self.env = saved_env;
                        cur = parent;
                        ip = rip;
                        self.stack.push(ret);
                    } else {
                        return Ok(ret);
                    }
                }
                Op::Call(argc) => {
                    let mut args = Vec::with_capacity(argc);

                    for _ in 0..argc {
                        args.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| "stack underflow".to_string())?,
                        );
                    }

                    args.reverse();

                    let callee = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;

                    match callee {
                        PyObject::Function(fobj) => {
                            if fobj.arity != argc {
                                return Err(format!(
                                    "TypeError: {}() expected {} args, got {}",
                                    fobj.name, fobj.arity, argc
                                ));
                            }

                            let mut new_env = Env::default();

                            for (i, name) in fobj.code.names.iter().take(argc).enumerate() {
                                new_env.locals.insert(name.clone(), args[i].clone());
                            }

                            new_env.globals = fobj.globals.clone().globals;
                            new_env.builtins = self.env.builtins.clone();
                            frames.push((
                                ip + 1,
                                cur.clone(),
                                std::mem::replace(&mut self.env, new_env),
                            ));
                            cur = fobj.code.clone();
                            ip = 0;
                        }
                        PyObject::NativeFunction(nf) => {
                            if nf.arity != usize::MAX && nf.arity != argc {
                                return Err(format!(
                                    "TypeError: {}() expected {} args, got {}",
                                    nf.name, nf.arity, argc
                                ));
                            }

                            let r = (nf.func)(&args)?;
                            self.stack.push(r);
                            ip += 1;
                        }
                        _ => return Err("TypeError: object not callable".to_string()),
                    }
                }
                Op::Def {
                    name,
                    arity,
                    code_idx,
                } => {
                    let fname = cur.names[name].clone();
                    let fcode = cur.nested[code_idx].clone();
                    let f = PyFunction {
                        name: fname.clone(),
                        arity,
                        code: fcode,
                        globals: self.env.clone(),
                    };

                    self.env
                        .locals
                        .insert(fname, PyObject::Function(Rc::new(f)));
                    ip += 1;
                }
                Op::Add => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(arith_add(a, b)?);
                    ip += 1;
                }
                Op::Sub => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(arith_sub(a, b)?);
                    ip += 1;
                }
                Op::Mul => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(arith_mul(a, b)?);
                    ip += 1;
                }
                Op::Div => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(arith_div(a, b)?);
                    ip += 1;
                }
                Op::Eq => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(PyObject::Bool(a == b));
                    ip += 1;
                }
                Op::Ne => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(PyObject::Bool(a != b));
                    ip += 1;
                }
                Op::Lt => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(cmp_lt(a, b)?);
                    ip += 1;
                }
                Op::Le => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(cmp_le(a, b)?);
                    ip += 1;
                }
                Op::Gt => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(cmp_gt(a, b)?);
                    ip += 1;
                }
                Op::Ge => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    self.stack.push(cmp_ge(a, b)?);
                    ip += 1;
                }
                Op::Jump(target) => {
                    ip = target;
                }
                Op::JumpIfFalse(target) => {
                    let v = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    if is_falsey(&v) {
                        ip = target;
                    } else {
                        ip += 1;
                    }
                }
            }
        }
    }
}

fn is_falsey(v: &PyObject) -> bool {
    match v {
        PyObject::Bool(b) => !b,
        PyObject::None => true,
        PyObject::Int(i) => *i == 0,
        PyObject::Float(x) => *x == 0.0,
        PyObject::Str(s) => s.is_empty(),
        _ => false,
    }
}

fn arith_add(a: PyObject, b: PyObject) -> Result<PyObject, String> {
    match (a, b) {
        (PyObject::Int(x), PyObject::Int(y)) => Ok(PyObject::Int(x + y)),
        (PyObject::Float(x), PyObject::Float(y)) => Ok(PyObject::Float(x + y)),
        (PyObject::Int(x), PyObject::Float(y)) => Ok(PyObject::Float(x as f64 + y)),
        (PyObject::Float(x), PyObject::Int(y)) => Ok(PyObject::Float(x + y as f64)),
        (PyObject::Str(a), PyObject::Str(b)) => Ok(PyObject::Str(a + &b)),
        _ => Err("TypeError: unsupported operand type(s) for +".to_string()),
    }
}

fn arith_sub(a: PyObject, b: PyObject) -> Result<PyObject, String> {
    match (a, b) {
        (PyObject::Int(x), PyObject::Int(y)) => Ok(PyObject::Int(x - y)),
        (PyObject::Float(x), PyObject::Float(y)) => Ok(PyObject::Float(x - y)),
        (PyObject::Int(x), PyObject::Float(y)) => Ok(PyObject::Float(x as f64 - y)),
        (PyObject::Float(x), PyObject::Int(y)) => Ok(PyObject::Float(x - y as f64)),
        _ => Err("TypeError: unsupported operand type(s) for -".to_string()),
    }
}

fn arith_mul(a: PyObject, b: PyObject) -> Result<PyObject, String> {
    match (a, b) {
        (PyObject::Int(x), PyObject::Int(y)) => Ok(PyObject::Int(x * y)),
        (PyObject::Float(x), PyObject::Float(y)) => Ok(PyObject::Float(x * y)),
        (PyObject::Int(x), PyObject::Float(y)) => Ok(PyObject::Float(x as f64 * y)),
        (PyObject::Float(x), PyObject::Int(y)) => Ok(PyObject::Float(x * y as f64)),
        _ => Err("TypeError: unsupported operand type(s) for *".to_string()),
    }
}

fn arith_div(a: PyObject, b: PyObject) -> Result<PyObject, String> {
    match (a, b) {
        (PyObject::Int(x), PyObject::Int(y)) => Ok(PyObject::Float(x as f64 / y as f64)),
        (PyObject::Float(x), PyObject::Float(y)) => Ok(PyObject::Float(x / y)),
        (PyObject::Int(x), PyObject::Float(y)) => Ok(PyObject::Float(x as f64 / y)),
        (PyObject::Float(x), PyObject::Int(y)) => Ok(PyObject::Float(x / y as f64)),
        _ => Err("TypeError: unsupported operand type(s) for /".to_string()),
    }
}

fn cmp_lt(a: PyObject, b: PyObject) -> Result<PyObject, String> {
    match (a, b) {
        (PyObject::Int(x), PyObject::Int(y)) => Ok(PyObject::Bool(x < y)),
        (PyObject::Float(x), PyObject::Float(y)) => Ok(PyObject::Bool(x < y)),
        (PyObject::Int(x), PyObject::Float(y)) => Ok(PyObject::Bool((x as f64) < y)),
        (PyObject::Float(x), PyObject::Int(y)) => Ok(PyObject::Bool(x < y as f64)),
        (PyObject::Str(a), PyObject::Str(b)) => Ok(PyObject::Bool(a < b)),
        _ => Err("TypeError: unsupported comparison".to_string()),
    }
}

fn cmp_le(a: PyObject, b: PyObject) -> Result<PyObject, String> {
    match (a, b) {
        (PyObject::Int(x), PyObject::Int(y)) => Ok(PyObject::Bool(x <= y)),
        (PyObject::Float(x), PyObject::Float(y)) => Ok(PyObject::Bool(x <= y)),
        (PyObject::Int(x), PyObject::Float(y)) => Ok(PyObject::Bool((x as f64) <= y)),
        (PyObject::Float(x), PyObject::Int(y)) => Ok(PyObject::Bool(x <= y as f64)),
        (PyObject::Str(a), PyObject::Str(b)) => Ok(PyObject::Bool(a <= b)),
        _ => Err("TypeError: unsupported comparison".to_string()),
    }
}

fn cmp_gt(a: PyObject, b: PyObject) -> Result<PyObject, String> {
    match (a, b) {
        (PyObject::Int(x), PyObject::Int(y)) => Ok(PyObject::Bool(x > y)),
        (PyObject::Float(x), PyObject::Float(y)) => Ok(PyObject::Bool(x > y)),
        (PyObject::Int(x), PyObject::Float(y)) => Ok(PyObject::Bool((x as f64) > y)),
        (PyObject::Float(x), PyObject::Int(y)) => Ok(PyObject::Bool(x > y as f64)),
        (PyObject::Str(a), PyObject::Str(b)) => Ok(PyObject::Bool(a > b)),
        _ => Err("TypeError: unsupported comparison".to_string()),
    }
}

fn cmp_ge(a: PyObject, b: PyObject) -> Result<PyObject, String> {
    match (a, b) {
        (PyObject::Int(x), PyObject::Int(y)) => Ok(PyObject::Bool(x >= y)),
        (PyObject::Float(x), PyObject::Float(y)) => Ok(PyObject::Bool(x >= y)),
        (PyObject::Int(x), PyObject::Float(y)) => Ok(PyObject::Bool((x as f64) >= y)),
        (PyObject::Float(x), PyObject::Int(y)) => Ok(PyObject::Bool(x >= y as f64)),
        (PyObject::Str(a), PyObject::Str(b)) => Ok(PyObject::Bool(a >= b)),
        _ => Err("TypeError: unsupported comparison".to_string()),
    }
}
