use crate::bytecode::*;
use crate::object::*;
use crate::opcode::*;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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
    pub loop_stack: Vec<(usize, usize)>,
    pub iter_stack: Vec<(usize, PyObject)>,
    pub modules: HashMap<String, PyObject>,
}

impl Vm {
    pub fn with_builtins(mut self) -> Self {
        self.env.builtins.insert(
            "set".to_string(),
            PyObject::NativeFunction(Rc::new(PyNativeFunction {
                name: "set".to_string(),
                arity: 0,
                func: Rc::new(|_| Ok(PyObject::Set(Rc::new(RefCell::new(HashSet::new()))))),
            })),
        );

        self.env.builtins.insert(
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
                            if let (PyObject::Int(start), PyObject::Int(stop)) =
                                (&args[0], &args[1])
                            {
                                (*start, *stop, 1)
                            } else {
                                return Err(
                                    "TypeError: range() arguments must be integers".to_string()
                                );
                            }
                        }
                        3 => {
                            if let (
                                PyObject::Int(start),
                                PyObject::Int(stop),
                                PyObject::Int(step),
                            ) = (&args[0], &args[1], &args[2])
                            {
                                if *step == 0 {
                                    return Err(
                                        "ValueError: range() arg 3 must not be zero".to_string()
                                    );
                                }
                                (*start, *stop, *step)
                            } else {
                                return Err(
                                    "TypeError: range() arguments must be integers".to_string()
                                );
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

        self
    }

    pub fn register_native_module(&mut self, name: &str, dict: HashMap<String, PyObject>) {
        let module = PyNativeModule {
            name: name.to_string(),
            dict,
        };
        self.modules
            .insert(name.to_string(), PyObject::NativeModule(Rc::new(module)));
    }

    pub fn register_native_class<F>(
        &mut self,
        name: &str,
        constructor: F,
        methods: HashMap<String, PyObject>,
    ) where
        F: Fn(&[PyObject]) -> Result<PyObject, String> + 'static,
    {
        let class = PyNativeClass {
            name: name.to_string(),
            methods,
            constructor: Rc::new(constructor),
        };

        let class_constructor = PyNativeFunction {
            name: name.to_string(),
            arity: usize::MAX,
            func: {
                let class_rc = Rc::new(class);
                Rc::new(move |args| (class_rc.constructor)(args))
            },
        };

        self.env.builtins.insert(
            name.to_string(),
            PyObject::NativeFunction(Rc::new(class_constructor)),
        );
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

    fn load_module(&mut self, name: &str) -> Result<PyObject, String> {
        if let Some(module) = self.modules.get(name) {
            return Ok(module.clone());
        }

        let filename = format!("{}.py", name);
        let source = std::fs::read_to_string(&filename)
            .map_err(|_| format!("ModuleNotFoundError: No module named '{}'", name))?;

        let mut compiler = crate::ast::Compiler::default();
        let code = compiler.compile(&source)?;

        let mut module_vm = Vm {
            stack: Vec::new(),
            env: Env::default(),
            loop_stack: Vec::new(),
            iter_stack: Vec::new(),
            modules: self.modules.clone(),
        }
        .with_builtins();

        module_vm.run(&code)?;

        let module = PyModule {
            name: name.to_string(),
            dict: module_vm.env.locals,
        };

        let module_obj = PyObject::Module(Rc::new(RefCell::new(module)));
        self.modules.insert(name.to_string(), module_obj.clone());

        Ok(module_obj)
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
                Op::UnaryNeg => {
                    let operand = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;

                    match operand {
                        PyObject::Int(x) => self.stack.push(PyObject::Int(-x)),
                        PyObject::Float(x) => self.stack.push(PyObject::Float(-x)),
                        _ => {
                            return Err(
                                "TypeError: unsupported operand type for unary -".to_string()
                            );
                        }
                    }

                    ip += 1;
                }
                Op::UnaryPos => {
                    let operand = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;

                    match operand {
                        PyObject::Int(x) => self.stack.push(PyObject::Int(x)),
                        PyObject::Float(x) => self.stack.push(PyObject::Float(x)),
                        _ => {
                            return Err(
                                "TypeError: unsupported operand type for unary +".to_string()
                            );
                        }
                    }

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
                Op::JumpIfTrue(target) => {
                    let v = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    if !is_falsey(&v) {
                        ip = target;
                    } else {
                        ip += 1;
                    }
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
                Op::SetupLoop(exit_addr) => {
                    self.loop_stack.push((ip + 1, exit_addr));
                    ip += 1;
                }
                Op::PopBlock => {
                    self.loop_stack.pop();
                    ip += 1;
                }
                Op::Break => {
                    if let Some((_, exit_addr)) = self.loop_stack.pop() {
                        ip = exit_addr;
                    } else {
                        return Err("SyntaxError: 'break' outside loop".to_string());
                    }
                }
                Op::Continue => {
                    if let Some((continue_addr, _)) = self.loop_stack.last() {
                        ip = *continue_addr;
                    } else {
                        return Err("SyntaxError: 'continue' not properly in loop".to_string());
                    }
                }
                Op::GetIter => {
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    match obj {
                        PyObject::List(l) => {
                            self.iter_stack.push((0, PyObject::List(l.clone())));
                            ip += 1;
                        }
                        PyObject::Tuple(t) => {
                            self.iter_stack.push((0, PyObject::Tuple(t.clone())));
                            ip += 1;
                        }
                        _ => return Err("TypeError: object is not iterable".to_string()),
                    }
                }
                Op::ForIter(exit_addr) => {
                    if let Some((index, iter_obj)) = self.iter_stack.last_mut() {
                        let has_next = match iter_obj {
                            PyObject::List(l) => {
                                let list = l.borrow();
                                if *index < list.len() {
                                    self.stack.push(list[*index].clone());
                                    *index += 1;
                                    true
                                } else {
                                    false
                                }
                            }
                            PyObject::Tuple(t) => {
                                if *index < t.len() {
                                    self.stack.push(t[*index].clone());
                                    *index += 1;
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        };

                        if has_next {
                            ip += 1;
                        } else {
                            self.iter_stack.pop();
                            ip = exit_addr;
                        }
                    } else {
                        return Err("RuntimeError: no iterator on stack".to_string());
                    }
                }
                Op::BuildList(count) => {
                    let mut items = Vec::with_capacity(count);

                    for _ in 0..count {
                        items.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| "stack underflow".to_string())?,
                        );
                    }

                    items.reverse();
                    self.stack
                        .push(PyObject::List(Rc::new(RefCell::new(items))));
                    ip += 1;
                }
                Op::BuildDict(count) => {
                    let mut pairs = Vec::new();

                    for _ in 0..count {
                        let value = self
                            .stack
                            .pop()
                            .ok_or_else(|| "stack underflow".to_string())?;
                        let key = self
                            .stack
                            .pop()
                            .ok_or_else(|| "stack underflow".to_string())?;
                        if let PyObject::Str(k) = key {
                            pairs.push((k, value));
                        } else {
                            return Err("TypeError: dict keys must be strings".to_string());
                        }
                    }

                    let mut dict = IndexMap::new();

                    for (k, v) in pairs.into_iter().rev() {
                        dict.insert(k, v);
                    }

                    self.stack.push(PyObject::Dict(Rc::new(RefCell::new(dict))));
                    ip += 1;
                }
                Op::LoadIndex => {
                    let index = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    match (obj, index) {
                        (PyObject::List(l), PyObject::Int(i)) => {
                            let list = l.borrow();
                            let idx = if i < 0 { list.len() as i64 + i } else { i } as usize;
                            if idx < list.len() {
                                self.stack.push(list[idx].clone());
                            } else {
                                return Err("IndexError: list index out of range".to_string());
                            }
                        }
                        (PyObject::Dict(d), PyObject::Str(k)) => {
                            if let Some(v) = d.borrow().get(&k) {
                                self.stack.push(v.clone());
                            } else {
                                return Err(format!("KeyError: '{}'", k));
                            }
                        }
                        (PyObject::Tuple(t), PyObject::Int(i)) => {
                            let idx = if i < 0 { t.len() as i64 + i } else { i } as usize;
                            if idx < t.len() {
                                self.stack.push(t[idx].clone());
                            } else {
                                return Err("IndexError: tuple index out of range".to_string());
                            }
                        }
                        _ => return Err("TypeError: invalid indexing operation".to_string()),
                    }

                    ip += 1;
                }
                Op::StoreIndex => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let index = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;

                    match (&obj, index) {
                        (PyObject::List(l), PyObject::Int(i)) => {
                            let mut list = l.borrow_mut();
                            let idx = if i < 0 { list.len() as i64 + i } else { i } as usize;
                            if idx < list.len() {
                                list[idx] = value;
                            } else {
                                return Err(
                                    "IndexError: list assignment index out of range".to_string()
                                );
                            }
                        }
                        (PyObject::Dict(d), PyObject::Str(k)) => {
                            d.borrow_mut().insert(k, value);
                        }
                        _ => return Err("TypeError: invalid indexing assignment".to_string()),
                    }

                    ip += 1;
                }
                Op::BuildTuple(count) => {
                    let mut items = Vec::with_capacity(count);

                    for _ in 0..count {
                        items.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| "stack underflow".to_string())?,
                        );
                    }

                    items.reverse();
                    self.stack.push(PyObject::Tuple(items));
                    ip += 1;
                }
                Op::BuildSet(count) => {
                    let mut set = std::collections::HashSet::new();

                    for _ in 0..count {
                        let item = self
                            .stack
                            .pop()
                            .ok_or_else(|| "stack underflow".to_string())?;
                        set.insert(item);
                    }

                    self.stack.push(PyObject::Set(Rc::new(RefCell::new(set))));
                    ip += 1;
                }
                Op::ClassDef { name, code_idx } => {
                    let class_name = cur.names[name].clone();
                    let class_code = cur.nested[code_idx].clone();

                    #[allow(unused_mut)]
                    let mut class_env = self.env.clone();
                    let mut class_vm = Vm {
                        stack: Vec::new(),
                        env: class_env,
                        loop_stack: Vec::new(),
                        iter_stack: Vec::new(),
                        ..Default::default()
                    };

                    class_vm.run(&class_code)?;

                    let mut methods = HashMap::new();

                    for (k, v) in class_vm.env.locals {
                        methods.insert(k, v);
                    }

                    let class = PyClass {
                        name: class_name.clone(),
                        methods,
                        bases: Vec::new(),
                    };

                    let constructor = PyNativeFunction {
                        name: class_name.clone(),
                        arity: usize::MAX,
                        func: {
                            let class_rc = Rc::new(class.clone());
                            Rc::new(move |args| {
                                let instance = PyInstance {
                                    class: class_rc.clone(),
                                    attrs: HashMap::new(),
                                };
                                let inst_obj = PyObject::Instance(Rc::new(RefCell::new(instance)));

                                if let Some(init_method) = class_rc.methods.get("__init__") {
                                    match init_method {
                                        PyObject::Function(f) => {
                                            let mut init_args = vec![inst_obj.clone()];
                                            init_args.extend_from_slice(args);

                                            let mut init_vm = Vm::default();
                                            let mut new_env = Env::default();

                                            for (i, name) in f
                                                .code
                                                .names
                                                .iter()
                                                .take(init_args.len())
                                                .enumerate()
                                            {
                                                new_env
                                                    .locals
                                                    .insert(name.clone(), init_args[i].clone());
                                            }

                                            new_env.globals = f.globals.clone().globals;
                                            init_vm.env = new_env;
                                            init_vm.run(&f.code)?;
                                        }
                                        _ => {}
                                    }
                                }

                                Ok(inst_obj)
                            })
                        },
                    };

                    self.env
                        .locals
                        .insert(class_name, PyObject::NativeFunction(Rc::new(constructor)));
                    ip += 1;
                }
                Op::LoadAttr(idx) => {
                    let attr_name = &cur.names[idx];
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;

                    match obj {
                        PyObject::Instance(inst) => {
                            let instance = inst.borrow();
                            if let Some(value) = instance.attrs.get(attr_name) {
                                self.stack.push(value.clone());
                            } else if let Some(method) = instance.class.methods.get(attr_name) {
                                match method {
                                    PyObject::Function(f) => {
                                        let bound_method = PyNativeFunction {
                                            name: format!("{}.{}", instance.class.name, attr_name),
                                            arity: f.arity - 1,
                                            func: {
                                                let f_clone = f.clone();
                                                let inst_clone = PyObject::Instance(inst.clone());
                                                Rc::new(move |args| {
                                                    let mut full_args = vec![inst_clone.clone()];
                                                    full_args.extend_from_slice(args);

                                                    let mut method_vm = Vm::default();
                                                    let mut new_env = Env::default();

                                                    for (i, name) in f_clone
                                                        .code
                                                        .names
                                                        .iter()
                                                        .take(full_args.len())
                                                        .enumerate()
                                                    {
                                                        new_env.locals.insert(
                                                            name.clone(),
                                                            full_args[i].clone(),
                                                        );
                                                    }

                                                    new_env.globals =
                                                        f_clone.globals.clone().globals;
                                                    method_vm.env = new_env;
                                                    method_vm.run(&f_clone.code)
                                                })
                                            },
                                        };
                                        self.stack
                                            .push(PyObject::NativeFunction(Rc::new(bound_method)));
                                    }
                                    _ => self.stack.push(method.clone()),
                                }
                            } else {
                                return Err(format!(
                                    "AttributeError: '{}' object has no attribute '{}'",
                                    instance.class.name, attr_name
                                ));
                            }
                        }
                        PyObject::Module(m) => {
                            let module = m.borrow();
                            if let Some(value) = module.dict.get(attr_name) {
                                self.stack.push(value.clone());
                            } else {
                                return Err(format!(
                                    "AttributeError: module '{}' has no attribute '{}'",
                                    module.name, attr_name
                                ));
                            }
                        }
                        PyObject::NativeModule(m) => {
                            if let Some(value) = m.dict.get(attr_name) {
                                self.stack.push(value.clone());
                            } else {
                                return Err(format!(
                                    "AttributeError: module '{}' has no attribute '{}'",
                                    m.name, attr_name
                                ));
                            }
                        }
                        PyObject::NativeClass(c) => {
                            if let Some(method) = c.methods.get(attr_name) {
                                self.stack.push(method.clone());
                            } else {
                                return Err(format!(
                                    "AttributeError: type '{}' has no attribute '{}'",
                                    c.name, attr_name
                                ));
                            }
                        }
                        _ => return Err("AttributeError: object has no attributes".to_string()),
                    }

                    ip += 1;
                }
                Op::StoreAttr(idx) => {
                    let attr_name = cur.names[idx].clone();
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;

                    match &obj {
                        PyObject::Instance(inst) => {
                            inst.borrow_mut().attrs.insert(attr_name, value);
                        }
                        _ => return Err("AttributeError: cannot set attribute".to_string()),
                    }

                    ip += 1;
                }
                Op::CallMethod(argc) => {
                    let mut args = Vec::with_capacity(argc);

                    for _ in 0..argc {
                        args.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| "stack underflow".to_string())?,
                        );
                    }

                    args.reverse();

                    let method = self
                        .stack
                        .pop()
                        .ok_or_else(|| "stack underflow".to_string())?;

                    match method {
                        PyObject::NativeFunction(nf) => {
                            let result = (nf.func)(&args)?;
                            self.stack.push(result);
                        }
                        _ => return Err("TypeError: object not callable".to_string()),
                    }

                    ip += 1;
                }
                Op::Import(idx) => {
                    let module_name = &cur.names[idx];
                    let module = self.load_module(module_name)?;
                    self.env.locals.insert(module_name.clone(), module);
                    ip += 1;
                }
                Op::ImportFrom { module, ref names } => {
                    let module_name = cur.names[module].clone();
                    let module_obj = self.load_module(&module_name)?;

                    match module_obj {
                        PyObject::Module(m) => {
                            let module_dict = &m.borrow().dict;
                            for name_idx in names {
                                let name = cur.names[*name_idx].clone();
                                if let Some(value) = module_dict.get(&name) {
                                    self.env.locals.insert(name.clone(), value.clone());
                                } else {
                                    return Err(format!(
                                        "ImportError: cannot import name '{}' from '{}'",
                                        name, module_name
                                    ));
                                }
                            }
                        }
                        PyObject::NativeModule(m) => {
                            for name_idx in names {
                                let name = cur.names[*name_idx].clone();
                                if let Some(value) = m.dict.get(&name) {
                                    self.env.locals.insert(name.clone(), value.clone());
                                } else {
                                    return Err(format!(
                                        "ImportError: cannot import name '{}' from '{}'",
                                        name, module_name
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }

                    ip += 1;
                }
                Op::ImportStar(idx) => {
                    let module_name = cur.names[idx].clone();
                    let module_obj = self.load_module(&module_name)?;

                    match module_obj {
                        PyObject::Module(m) => {
                            let module_dict = &m.borrow().dict;
                            for (name, value) in module_dict {
                                if !name.starts_with('_') {
                                    self.env.locals.insert(name.clone(), value.clone());
                                }
                            }
                        }
                        PyObject::NativeModule(m) => {
                            for (name, value) in &m.dict {
                                if !name.starts_with('_') {
                                    self.env.locals.insert(name.clone(), value.clone());
                                }
                            }
                        }
                        _ => {}
                    }

                    ip += 1;
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
        PyObject::List(l) => l.borrow().is_empty(),
        PyObject::Dict(d) => d.borrow().is_empty(),
        PyObject::Tuple(t) => t.is_empty(),
        PyObject::Set(s) => s.borrow().is_empty(),
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
