use crate::bytecode::*;
use crate::object::*;
use crate::opcode::*;
use std::collections::BTreeMap;

use ruff_python_ast::{self as ast, Mod};
use ruff_python_parser::{Mode, ParseOptions, parse};

#[derive(Default)]
pub struct Compiler {
    #[allow(dead_code)]
    pub strings: BTreeMap<String, usize>,
}

impl Compiler {
    pub fn compile(&mut self, source: &str) -> Result<CodeObject, String> {
        let module = parse(source, ParseOptions::from(Mode::Module)).map_err(|e| e.to_string())?;
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
