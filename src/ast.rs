#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Dynamic, // Default when omitted
    Named(String),
    Reference {
        mutable: bool,
        inner: Box<TypeAnnotation>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: TypeAnnotation,
    pub is_self: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralValue),
    Var(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    IndexAccess {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Borrow {
        mutable: bool,
        expr: Box<Expr>,
    },
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Import {
        module: Vec<String>,
        alias: Option<String>,
    },
    Def {
        name: String,
        params: Vec<Parameter>,
        return_type: TypeAnnotation,
        body: Vec<Stmt>,
    },
    Class {
        name: String,
        fields: Vec<(String, TypeAnnotation)>,
    },
    Protocol {
        name: String,
        methods: Vec<(String, Vec<Parameter>, TypeAnnotation)>,
    },
    Impl {
        target: String,
        protocol: Option<String>,
        methods: Vec<Stmt>,
    },
    Let {
        name: String,
        is_mut: bool,
        ty: TypeAnnotation,
        init: Expr,
    },
    Assignment {
        target: Expr,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    ExprStmt(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
