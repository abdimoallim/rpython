use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Display};
use std::rc::Rc;

use ruff_python_ast::{self as ast, Mod};
use ruff_python_parser::{Mode, ParseOptions, parse};

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

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    LoadConst(usize),
    LoadName(usize),
    StoreName(usize),
    LoadGlobal(usize),
    StoreGlobal(usize),
    Pop,
    Return,
    Call(usize),
    Def {
        name: usize,
        arity: usize,
        code_idx: usize,
    },
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Jump(usize),
    JumpIfFalse(usize),
}

impl Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::LoadConst(idx) => write!(f, "LoadConst({})", idx),
            Op::LoadName(idx) => write!(f, "LoadName({})", idx),
            Op::StoreName(idx) => write!(f, "StoreName({})", idx),
            Op::LoadGlobal(idx) => write!(f, "LoadGlobal({})", idx),
            Op::StoreGlobal(idx) => write!(f, "StoreGlobal({})", idx),
            Op::Pop => write!(f, "Pop"),
            Op::Return => write!(f, "Return"),
            Op::Call(argc) => write!(f, "Call({})", argc),
            Op::Def {
                name,
                arity,
                code_idx,
            } => write!(
                f,
                "Def(name={}, arity={}, code_idx={})",
                name, arity, code_idx
            ),
            Op::Add => write!(f, "Add"),
            Op::Sub => write!(f, "Sub"),
            Op::Mul => write!(f, "Mul"),
            Op::Div => write!(f, "Div"),
            Op::Eq => write!(f, "Eq"),
            Op::Ne => write!(f, "Ne"),
            Op::Lt => write!(f, "Lt"),
            Op::Le => write!(f, "Le"),
            Op::Gt => write!(f, "Gt"),
            Op::Ge => write!(f, "Ge"),
            Op::Jump(target) => write!(f, "Jump({})", target),
            Op::JumpIfFalse(target) => write!(f, "JumpIfFalse({})", target),
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct CodeObject {
    pub consts: Vec<PyObject>,
    pub names: Vec<String>,
    pub instructions: Vec<Op>,
    pub nested: Vec<CodeObject>,
}

impl CodeObject {
    pub fn debug_print(&self) {
        println!("Constants: {:?}", self.consts);
        println!("Names: {:?}", self.names);
        println!("Instructions:");

        for (i, op) in self.instructions.iter().enumerate() {
            println!("  {}: {}", i, op);
        }
    }
}

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

#[derive(Default)]
pub struct Compiler {
    pub strings: BTreeMap<String, usize>,
}

impl Compiler {
    pub fn compile(&mut self, source: &str) -> Result<CodeObject, String> {
        let module = parse(source, ParseOptions::from(Mode::Module)).map_err(|e| e.to_string())?;
        // let module = module.try_into_module().unwrap();
        let module = match module.syntax() {
            Mod::Module(module) => module,
            Mod::Expression(_) => return Err("Invalid syntax".to_string()),
        };
        let module = ruff_python_ast::Mod::Module(module.clone());
        let mut code = CodeObject::default();
        self.compile_body(&module, &mut code)?;
        Ok(code)
    }

    fn name_index(&mut self, code: &mut CodeObject, name: &str) -> usize {
        if let Some((i, _)) = code.names.iter().enumerate().find(|(_, n)| n == &name) {
            i
        } else {
            code.names.push(name.to_string());
            code.names.len() - 1
        }
    }

    fn const_index(&mut self, code: &mut CodeObject, obj: PyObject) -> usize {
        if let Some((i, _)) = code.consts.iter().enumerate().find(|(_, v)| *v == &obj) {
            i
        } else {
            code.consts.push(obj);
            code.consts.len() - 1
        }
    }

    fn compile_body(&mut self, module: &ast::Mod, code: &mut CodeObject) -> Result<(), String> {
        match module {
            ast::Mod::Module(ast::ModModule { body, .. }) => {
                for stmt in body {
                    self.compile_stmt(stmt, code)?;
                }
                // code.instructions.push(Op::LoadConst(
                //     self.const_index(&mut code.clone(), PyObject::None),
                // ));
                // let none_idx = self.const_index(code, PyObject::None);
                // code.instructions.push(Op::LoadConst(none_idx));

                if body.is_empty() {
                    let none_idx = self.const_index(code, PyObject::None);
                    code.instructions.push(Op::LoadConst(none_idx));
                }

                code.instructions.push(Op::Return);
                Ok(())
            }
            _ => Err("unsupported mode".to_string()),
        }
    }

    fn compile_stmt(&mut self, stmt: &ast::Stmt, code: &mut CodeObject) -> Result<(), String> {
        match stmt {
            ast::Stmt::Assign(a) => {
                if a.targets.len() != 1 {
                    return Err("unsupported assignment".to_string());
                }
                self.compile_expr(&a.value, code)?;
                if let ast::Expr::Name(n) = &a.targets[0] {
                    let idx = self.name_index(code, n.id.as_str());
                    code.instructions.push(Op::StoreName(idx));
                    Ok(())
                } else {
                    Err("unsupported assignment target".to_string())
                }
            }
            ast::Stmt::Expr(e) => {
                self.compile_expr(&e.value, code)?;
                // code.instructions.push(Op::Pop);
                Ok(())
            }
            ast::Stmt::FunctionDef(fd) => {
                let mut fcode = CodeObject::default();
                let mut arg_names = Vec::new();
                for arg in &fd.parameters.args {
                    arg_names.push(arg.parameter.name.to_string());
                }
                for a in &arg_names {
                    self.name_index(&mut fcode, a);
                }
                for s in &fd.body {
                    self.compile_stmt(s, &mut fcode)?;
                }
                // fcode.instructions.push(Op::LoadConst(
                //     self.const_index(&mut fcode.clone(), PyObject::None),
                // ));
                let none_idx = self.const_index(&mut fcode, PyObject::None);
                fcode.instructions.push(Op::LoadConst(none_idx));
                // fcode.instructions.push(Op::Return);
                let code_idx = code.nested.len();
                code.nested.push(fcode);
                let name_idx = self.name_index(code, fd.name.as_str());
                let arity = arg_names.len();
                code.instructions.push(Op::Def {
                    name: name_idx,
                    arity,
                    code_idx,
                });
                // code.instructions.push(Op::StoreName(name_idx));
                Ok(())
            }
            ast::Stmt::Return(ret) => {
                if let Some(value) = &ret.value {
                    self.compile_expr(value, code)?;
                } else {
                    let none_idx = self.const_index(code, PyObject::None);
                    code.instructions.push(Op::LoadConst(none_idx));
                }

                code.instructions.push(Op::Return);
                Ok(())
            }
            _ => Err("unsupported statement".to_string()),
        }
    }

    fn compile_expr(&mut self, expr: &ast::Expr, code: &mut CodeObject) -> Result<(), String> {
        match expr {
            ast::Expr::BooleanLiteral(bl) => {
                let obj = PyObject::Bool(bl.value);
                let idx = self.const_index(code, obj);
                code.instructions.push(Op::LoadConst(idx));
                Ok(())
            }
            ast::Expr::StringLiteral(sl) => {
                let obj = PyObject::Str(sl.value.to_string());
                let idx = self.const_index(code, obj);
                code.instructions.push(Op::LoadConst(idx));
                Ok(())
            }
            ast::Expr::NumberLiteral(il) => {
                let obj = if il.value.is_int() {
                    PyObject::Int(il.value.as_int().unwrap().as_i64().unwrap())
                } else {
                    PyObject::Float(*il.value.as_float().unwrap())
                };
                let idx = self.const_index(code, obj);
                code.instructions.push(Op::LoadConst(idx));
                Ok(())
            }
            ast::Expr::NoneLiteral(_) => {
                let obj = PyObject::None;
                let idx = self.const_index(code, obj);
                code.instructions.push(Op::LoadConst(idx));
                Ok(())
            }
            ast::Expr::Name(n) => {
                let idx = self.name_index(code, n.id.as_str());
                code.instructions.push(Op::LoadName(idx));
                Ok(())
            }
            ast::Expr::BinOp(b) => {
                self.compile_expr(&b.left, code)?;
                self.compile_expr(&b.right, code)?;

                match b.op {
                    ast::Operator::Add => code.instructions.push(Op::Add),
                    ast::Operator::Sub => code.instructions.push(Op::Sub),
                    ast::Operator::Mult => code.instructions.push(Op::Mul),
                    ast::Operator::Div => code.instructions.push(Op::Div),
                    _ => return Err("unsupported binop".to_string()),
                }

                Ok(())
            }
            ast::Expr::Compare(cmp) => {
                if cmp.ops.len() != 1 || cmp.comparators.len() != 1 {
                    return Err("unsupported comparison".to_string());
                }

                self.compile_expr(&cmp.left, code)?;
                self.compile_expr(&cmp.comparators[0], code)?;

                match cmp.ops[0] {
                    ast::CmpOp::Eq => code.instructions.push(Op::Eq),
                    ast::CmpOp::NotEq => code.instructions.push(Op::Ne),
                    ast::CmpOp::Lt => code.instructions.push(Op::Lt),
                    ast::CmpOp::LtE => code.instructions.push(Op::Le),
                    ast::CmpOp::Gt => code.instructions.push(Op::Gt),
                    ast::CmpOp::GtE => code.instructions.push(Op::Ge),
                    _ => return Err("unsupported comparison".to_string()),
                }

                Ok(())
            }
            ast::Expr::Call(call) => {
                self.compile_expr(&call.func, code)?;

                let argc = call.arguments.len();
                // let argc = call.arguments.args.len();

                for a in &call.arguments.args {
                    self.compile_expr(a, code)?;
                }

                code.instructions.push(Op::Call(argc));

                Ok(())
            }
            _ => Err("unsupported expression".to_string()),
        }
    }
}

pub fn execute(
    source: &str,
    natives: &[(
        &str,
        usize,
        Rc<dyn Fn(&[PyObject]) -> Result<PyObject, String>>,
    )],
) -> Result<PyObject, String> {
    let mut compiler = Compiler::default();
    let code = compiler.compile(source)?;
    let mut vm = Vm::default().with_builtins();

    for (name, arity, f) in natives {
        vm.register_native(name, *arity, {
            let f = f.clone();
            move |args| f(args)
        });
    }

    vm.run(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let _ = execute("", &[]).unwrap();
    }

    #[test]
    fn expr() {
        let n = execute("2", &[]).unwrap();
        dbg!("found:", n.to_string());
    }

    #[test]
    fn basic_arith() {
        let r = execute("x=1+2\nx", &[]).unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    #[test]
    fn function_call() {
        let r = execute("def add(a,b):\n  return a+b\nadd(2,3)", &[]).unwrap();
        assert_eq!(format!("{}", r), "5");
    }

    #[test]
    fn native_fn() {
        let f = Rc::new(|args: &[PyObject]| -> Result<PyObject, String> {
            if let (PyObject::Int(a), PyObject::Int(b)) = (&args[0], &args[1]) {
                Ok(PyObject::Int(a + b))
            } else {
                Err("bad args".to_string())
            }
        });
        let r = execute("sum2(10,32)", &[("sum2", 2, f)]).unwrap();
        assert_eq!(format!("{}", r), "42");
    }
}
