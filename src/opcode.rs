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
    BuildTuple(usize),
    BuildSet(usize),
    LoadIndex,
    StoreIndex,
    Def {
        name: usize,
        arity: usize,
        code_idx: usize,
    },
    UnaryNeg,
    // ??
    UnaryPos,
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
    SetupLoop(usize),
    PopBlock,
    Break,
    Continue,
    ForIter(usize),
    GetIter,
    ClassDef {
        name: usize,
        code_idx: usize,
    },
    LoadAttr(usize),
    StoreAttr(usize),
    CallMethod(usize),
    Import(usize),
    ImportFrom {
        module: usize,
        names: Vec<usize>,
    },
    ImportStar(usize),
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
            Op::BuildTuple(count) => write!(f, "BuildTuple({})", count),
            Op::BuildSet(count) => write!(f, "BuildSet({})", count),
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
            Op::UnaryNeg => write!(f, "UnaryMinus"),
            Op::UnaryPos => write!(f, "UnaryPlus"),
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
            Op::SetupLoop(exit) => write!(f, "SetupLoop({})", exit),
            Op::PopBlock => write!(f, "PopBlock"),
            Op::Break => write!(f, "Break"),
            Op::Continue => write!(f, "Continue"),
            Op::GetIter => write!(f, "GetIter"),
            Op::ForIter(exit) => write!(f, "ForIter({})", exit),
            Op::ClassDef { name, code_idx } => {
                write!(f, "ClassDef(name={}, code_idx={})", name, code_idx)
            }
            Op::LoadAttr(idx) => write!(f, "LoadAttr({})", idx),
            Op::StoreAttr(idx) => write!(f, "StoreAttr({})", idx),
            Op::CallMethod(argc) => write!(f, "CallMethod({})", argc),
            Op::Import(idx) => write!(f, "Import({})", idx),
            Op::ImportFrom { module, names } => {
                write!(f, "ImportFrom(module={}, names={:?})", module, names)
            }
            Op::ImportStar(idx) => write!(f, "ImportStar({})", idx),
        }
    }
}
