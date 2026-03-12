#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Ptr,
    Struct(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLit(i32),
    FloatLit(f64),
    StringLit(String),
    Var(String),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Call(String, Vec<Expr>),
    FieldAccess(Box<Expr>, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let(String, Type, Option<Expr>),
    Assign(String, Expr),
    FieldAssign(String, String, Expr),
    Return(Expr),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub structs: Vec<StructDef>,
    pub functions: Vec<Function>,
}
