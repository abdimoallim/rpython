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
        let n = execute("2.3", &[]).unwrap();
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

    #[test]
    fn list_creation() {
        let r = execute("[1, 2, 3]", &[]).unwrap();
        assert_eq!(format!("{}", r), "[1, 2, 3]");
    }

    #[test]
    fn empty_list() {
        let r = execute("[]", &[]).unwrap();
        assert_eq!(format!("{}", r), "[]");
    }

    #[test]
    fn list_indexing() {
        let r = execute("x = [10, 20, 30]\nx[1]", &[]).unwrap();
        assert_eq!(format!("{}", r), "20");
    }

    #[test]
    fn list_negative_index() {
        let r = execute("x = [1, 2, 3]\nx[-1]", &[]).unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    // @todo: revisit ordering

    #[test]
    fn dict_creation() {
        let r = execute("{'a': 1, 'b': 2}", &[]).unwrap();
        assert_eq!(format!("{}", r), "{'a': 1, 'b': 2}");
    }

    #[test]
    fn empty_dict() {
        let r = execute("{}", &[]).unwrap();
        assert_eq!(format!("{}", r), "{}");
    }

    #[test]
    fn dict_access() {
        let r = execute("x = {'name': 'Alice', 'age': 30}\nx['name']", &[]).unwrap();
        assert_eq!(format!("{}", r), "Alice");
    }

    #[test]
    fn list_assignment() {
        let r = execute("x = [1, 2, 3]\nx[1] = 99\nx[1]", &[]).unwrap();
        assert_eq!(format!("{}", r), "99");
    }

    #[test]
    fn dict_assignment() {
        let r = execute("x = {'a': 1}\nx['b'] = 2\nx['b']", &[]).unwrap();
        assert_eq!(format!("{}", r), "2");
    }

    #[test]
    fn nested_structures() {
        let r = execute("x = [{'a': 1}, {'b': 2}]\nx[0]['a']", &[]).unwrap();
        assert_eq!(format!("{}", r), "1");
    }

    #[test]
    fn tuple_creation() {
        let r = execute("(1, 2, 3)", &[]).unwrap();
        assert_eq!(format!("{}", r), "(1, 2, 3)");
    }

    #[test]
    fn empty_tuple() {
        let r = execute("()", &[]).unwrap();
        assert_eq!(format!("{}", r), "()");
    }

    #[test]
    fn single_tuple() {
        let r = execute("(7,)", &[]).unwrap();
        assert_eq!(format!("{}", r), "(7,)");
    }

    #[test]
    fn tuple_indexing() {
        let r = execute("x = (10, 20, 30)\nx[1]", &[]).unwrap();
        assert_eq!(format!("{}", r), "20");
    }

    // @fix: unary/negative integers

    #[test]
    fn tuple_negative_index() {
        let r = execute("x = (1, 2, 3)\nx[-1]", &[]).unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    #[test]
    fn set_creation() {
        let r = execute("{1, 2, 3}", &[]).unwrap();
        let result = format!("{}", r);
        assert!(result.contains("1") && result.contains("2") && result.contains("3"));
    }

    // @todo: should probably be a set class

    #[test]
    fn empty_set() {
        let r = execute("set()", &[]).unwrap();
        assert_eq!(format!("{}", r), "{}");
    }

    #[test]
    fn set_deduplication() {
        let r = execute("{1, 2, 2, 3, 1}", &[]).unwrap();
        let result = format!("{}", r);
        assert!(result.len() < 15);
    }

    #[test]
    fn nested_tuple_list() {
        let r = execute("x = ([1, 2], (3, 4))\nx[1][0]", &[]).unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    #[test]
    fn mixed_types() {
        let r = execute("(1, 'hello', [2, 3])", &[]).unwrap();
        assert_eq!(format!("{}", r), "(1, hello, [2, 3])");
    }

    #[test]
    fn while_loop() {
        let r = execute("x = 0\nwhile x < 3:\n  x = x + 1\nx", &[]).unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    #[test]
    fn while_false() {
        let r = execute("x = 5\nwhile False:\n  x = 10\nx", &[]).unwrap();
        assert_eq!(format!("{}", r), "5");
    }

    #[test]
    fn while_with_break() {
        let r = execute(
            "x = 0\nwhile True:\n  x = x + 1\n  if x == 3:\n    break\nx",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    #[test]
    fn while_with_continue() {
        let r = execute(
            "x = 0\ny = 0\nwhile x < 5:\n  x = x + 1\n  if x == 3:\n    continue\n  y = y + 1\ny",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "4");
    }

    #[test]
    fn nested_while() {
        let r = execute("x = 0\ny = 0\nwhile x < 2:\n  x = x + 1\n  z = 0\n  while z < 2:\n    z = z + 1\n    y = y + 1\ny", &[]).unwrap();
        assert_eq!(format!("{}", r), "4");
    }

    #[test]
    fn while_accumulator() {
        let r = execute(
            "i = 1\nsum = 0\nwhile i <= 4:\n  sum = sum + i\n  i = i + 1\nsum",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "10");
    }

    #[test]
    fn for_loop_range() {
        let r = execute("sum = 0\nfor i in range(3):\n  sum = sum + i\nsum", &[]).unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    #[test]
    fn for_loop_range_start_stop() {
        let r = execute("sum = 0\nfor i in range(2, 5):\n  sum = sum + i\nsum", &[]).unwrap();
        assert_eq!(format!("{}", r), "9");
    }

    #[test]
    fn for_loop_range_step() {
        let r = execute(
            "sum = 0\nfor i in range(1, 8, 2):\n  sum = sum + i\nsum",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "16");
    }

    #[test]
    fn for_loop_list() {
        let r = execute("sum = 0\nfor x in [1, 2, 3]:\n  sum = sum + x\nsum", &[]).unwrap();
        assert_eq!(format!("{}", r), "6");
    }

    #[test]
    fn for_loop_tuple() {
        let r = execute("sum = 0\nfor x in (10, 20, 30):\n  sum = sum + x\nsum", &[]).unwrap();
        assert_eq!(format!("{}", r), "60");
    }

    #[test]
    fn for_loop_empty() {
        let r = execute("x = 5\nfor i in []:\n  x = 10\nx", &[]).unwrap();
        assert_eq!(format!("{}", r), "5");
    }

    #[test]
    fn for_loop_ident() {
        let r = execute("x = 5\ny=[10,20,30]\nfor i in y:\n  x = x + i\nx", &[]).unwrap();
        assert_eq!(format!("{}", r), "65");
    }

    #[test]
    fn for_loop_nested() {
        let r = execute(
            "sum = 0\nfor i in range(2):\n  for j in range(2):\n    sum = sum + 1\nsum",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "4");
    }

    #[test]
    fn for_loop_with_break() {
        let r = execute(
            "sum = 0\nfor i in range(10):\n  if i == 3:\n    break\n  sum = sum + i\nsum",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "3");
    }

    #[test]
    fn for_loop_with_continue() {
        let r = execute(
            "sum = 0\nfor i in range(5):\n  if i == 2:\n    continue\n  sum = sum + i\nsum",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "8");
    }

    #[test]
    fn range_negative_step() {
        let r = execute(
            "sum = 0\nfor i in range(5, 0, -1):\n  sum = sum + i\nsum",
            &[],
        )
        .unwrap();
        assert_eq!(format!("{}", r), "15");
    }
}
