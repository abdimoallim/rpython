use std::fmt::{self, Display};

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
    BuildList(usize),
    BuildDict(usize),
    LoadIndex,
    StoreIndex,
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
    JumpIfTrue(usize),
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
            Op::BuildList(count) => write!(f, "BuildList({})", count),
            Op::BuildDict(count) => write!(f, "BuildDict({})", count),
            Op::LoadIndex => write!(f, "LoadIndex"),
            Op::StoreIndex => write!(f, "StoreIndex"),
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
            Op::JumpIfTrue(target) => write!(f, "JumpIfTrue({})", target),
            Op::JumpIfFalse(target) => write!(f, "JumpIfFalse({})", target),
        }
    }
}
