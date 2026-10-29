# Lingo Grammar & Syntax Specification

## Lexical Grammar
```ebnf
IDENTIFIER  ::= [a-zA-Z_][a-zA-Z0-9_]*
INT_LIT     ::= [0-9]+ | "0x" [0-9a-fA-F]+ | "0b" [01]+
FLOAT_LIT   ::= [0-9]+ "." [0-9]+ ([eE][+-]?[0-9]+)?
STRING_LIT  ::= '"' ( [^"\\] | '\\' . )* '"'
COMMENT     ::= '#' [^\n]*
```

## Statement Grammar
```ebnf
Program        ::= Stmt*
Stmt           ::= FunctionDef | ClassDef | LetStmt | AssignStmt | IfStmt | WhileStmt | ForStmt | ReturnStmt | ExprStmt
FunctionDef    ::= "def" IDENTIFIER "(" ParamList? ")" ( "->" TypeAnnotation )? ":" Block
LetStmt        ::= "let" ( "mut" )? IDENTIFIER ( ":" TypeAnnotation )? "=" Expr
AssignStmt     ::= PrimaryExpr "=" Expr
IfStmt         ::= "if" Expr ":" Block ( "elif" Expr ":" Block )* ( "else" ":" Block )?
WhileStmt      ::= "while" Expr ":" Block
ForStmt        ::= "for" IDENTIFIER "in" Expr ":" Block
ReturnStmt     ::= "return" Expr?
Block          ::= NEWLINE INDENT Stmt+ DEDENT
```

## Expression Grammar (Operator Precedence)
1. Primary / Literal / Identifier / Call / Field Access (`.`) / Index (`[]`)
2. Unary: `-`, `not`
3. Multiplicative: `*`, `/`, `%`
4. Additive: `+`, `-`
5. Comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
6. Logical AND: `and`
7. Logical OR: `or`
