mod ast;
mod bytecode;
mod object;
mod opcode;
mod vm;

use std::rc::Rc;

use ast::*;
use object::*;
use vm::*;

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
        let r = execute(include_str!("../test/arith.py"), &[]).unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    #[test]
    fn function_call() {
        let r = execute(include_str!("../test/call.py"), &[]).unwrap();
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
        let r = execute("sum2(7,2)", &[("sum2", 2, f)]).unwrap();
        assert_eq!(format!("{}", r), "9");
    }

    #[test]
    fn builtins() {
        let r = execute(include_str!("../test/builtin.py"), &[]).unwrap();
        assert_eq!(format!("{}", r), "<type int>");
    }

    #[test]
    fn if_true() {
        let r = execute(include_str!("../test/if_true.py"), &[]).unwrap();
        assert_eq!(format!("{}", r), "5");
    }

    #[test]
    fn if_false() {
        let r = execute(include_str!("../test/if_false.py"), &[]).unwrap();
        assert_eq!(format!("{}", r), "10");
    }

    #[test]
    fn if_no_else() {
        let r = execute(include_str!("../test/if_no_else.py"), &[]).unwrap();
        assert_eq!(format!("{}", r), "2");
    }

    #[test]
    fn if_condition() {
        let r = execute(include_str!("../test/if_cond.py"), &[]).unwrap();
        assert_eq!(format!("{}", r), "100");
    }

    #[test]
    fn nested_if() {
        let r = execute(include_str!("../test/if_nested.py"), &[]).unwrap();
        assert_eq!(format!("{}", r), "2");
    }
}
