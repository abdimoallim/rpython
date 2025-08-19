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
        let r = execute("sum2(7,2)", &[("sum2", 2, f)]).unwrap();
        assert_eq!(format!("{}", r), "9");
    }

    #[test]
    fn builtins() {
        let r = execute("print(9)\ntype(9)", &[]).unwrap();
        assert_eq!(format!("{}", r), "<type int>");
    }

    #[test]
    fn if_true() {
        let r = execute("if False:\n  x = 5\nelse:\n  x = 10\nx", &[]).unwrap();
        assert_eq!(format!("{}", r), "5");
    }

    #[test]
    fn if_false() {
        let r = execute("if False:\n  x = 5\nelse:\n  x = 10\nx", &[]).unwrap();
        assert_eq!(format!("{}", r), "10");
    }

    #[test]
    fn if_no_else() {
        let r = execute("x = 1\nif True:\n  x = 2\nx", &[]).unwrap();
        assert_eq!(format!("{}", r), "2");
    }

    #[test]
    fn if_condition() {
        let r = execute("x = 5\nif x > 3:\n  y = 100\nelse:\n  y = 200\ny", &[]).unwrap();
        assert_eq!(format!("{}", r), "100");
    }

    #[test]
    fn nested_if() {
        let r = execute(
            "x = 5\nif x > 0:\n  if x > 10:\n    y = 1\n  else:\n    y = 2\nelse:\n  y = 3\ny",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "2");
    }
}
