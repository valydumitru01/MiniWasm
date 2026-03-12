use crate::c_compiler::ast::{Expr, Function, Op, Program, Stmt, StructDef, Type};
use crate::c_compiler::lexer::Token;

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_at(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: Token) {
        let tok = self.advance();
        assert_eq!(tok, expected, "unexpected token");
    }

    fn expect_ident(&mut self) -> String {
        match self.advance() {
            Token::Identifier(name) => name,
            other => panic!("expected identifier, got {:?}", other),
        }
    }

    fn is_type_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::KwInt | Token::KwFloat | Token::KwChar | Token::KwStruct)
        )
    }

    // ── Top-level ──────────────────────────────────────────────

    fn parse_program(&mut self) -> Program {
        let mut structs = Vec::new();
        let mut functions = Vec::new();

        while self.pos < self.tokens.len() {
            // struct Name { ... }; → struct definition
            // struct Name ident(  → function returning struct
            if matches!(self.peek(), Some(Token::KwStruct))
                && matches!(self.peek_at(2), Some(Token::LBrace))
            {
                structs.push(self.parse_struct_def());
            } else {
                functions.push(self.parse_function());
            }
        }

        Program { structs, functions }
    }

    fn parse_struct_def(&mut self) -> StructDef {
        self.expect(Token::KwStruct);
        let name = self.expect_ident();
        self.expect(Token::LBrace);

        let mut fields = Vec::new();
        while !matches!(self.peek(), Some(Token::RBrace)) {
            let ty = self.parse_type();
            let field_name = self.expect_ident();
            self.expect(Token::Semicolon);
            fields.push((field_name, ty));
        }
        self.expect(Token::RBrace);
        self.expect(Token::Semicolon);

        StructDef { name, fields }
    }

    fn parse_function(&mut self) -> Function {
        let return_type = self.parse_type();
        let name = self.expect_ident();
        self.expect(Token::LParen);

        let mut params = Vec::new();
        if !matches!(self.peek(), Some(Token::RParen)) {
            let ty = self.parse_type();
            let pname = self.expect_ident();
            params.push((pname, ty));
            while matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
                let ty = self.parse_type();
                let pname = self.expect_ident();
                params.push((pname, ty));
            }
        }
        self.expect(Token::RParen);
        self.expect(Token::LBrace);

        let mut body = Vec::new();
        while !matches!(self.peek(), Some(Token::RBrace)) {
            body.push(self.parse_statement());
        }
        self.expect(Token::RBrace);

        Function {
            name,
            params,
            return_type,
            body,
        }
    }

    fn parse_type(&mut self) -> Type {
        match self.advance() {
            Token::KwInt => Type::Int,
            Token::KwFloat => Type::Float,
            Token::KwChar => Type::Ptr,
            Token::KwStruct => {
                let name = self.expect_ident();
                Type::Struct(name)
            }
            other => panic!("expected type, got {:?}", other),
        }
    }

    // ── Statements ─────────────────────────────────────────────

    fn parse_statement(&mut self) -> Stmt {
        // return expr ;
        if matches!(self.peek(), Some(Token::KwReturn)) {
            self.advance();
            let expr = self.parse_expression();
            self.expect(Token::Semicolon);
            return Stmt::Return(expr);
        }

        // type name (= expr)? ;
        if self.is_type_start() {
            let ty = self.parse_type();
            let name = self.expect_ident();
            if matches!(self.peek(), Some(Token::Equal)) {
                self.advance();
                let expr = self.parse_expression();
                self.expect(Token::Semicolon);
                return Stmt::Let(name, ty, Some(expr));
            } else {
                self.expect(Token::Semicolon);
                return Stmt::Let(name, ty, None);
            }
        }

        // expression, then check for = (assignment) or ; (expr stmt)
        let expr = self.parse_expression();
        if matches!(self.peek(), Some(Token::Equal)) {
            self.advance();
            let value = self.parse_expression();
            self.expect(Token::Semicolon);
            match expr {
                Expr::Var(name) => Stmt::Assign(name, value),
                Expr::FieldAccess(obj, field) => match *obj {
                    Expr::Var(var) => Stmt::FieldAssign(var, field, value),
                    _ => panic!("unsupported assignment target"),
                },
                _ => panic!("invalid assignment target"),
            }
        } else {
            self.expect(Token::Semicolon);
            Stmt::Expr(expr)
        }
    }

    // ── Expressions (precedence climbing) ──────────────────────

    fn parse_expression(&mut self) -> Expr {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => Op::Add,
                Some(Token::Minus) => Op::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative();
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_primary();
        loop {
            let op = match self.peek() {
                Some(Token::Star) => Op::Mul,
                Some(Token::Slash) => Op::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_primary();
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_primary(&mut self) -> Expr {
        let tok = self.advance();
        let mut expr = match tok {
            Token::Int(n) => Expr::IntLit(n),
            Token::Float(f) => Expr::FloatLit(f),
            Token::String(s) => Expr::StringLit(s),
            Token::Identifier(name) => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.advance(); // (
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(Token::RParen)) {
                        args.push(self.parse_expression());
                        while matches!(self.peek(), Some(Token::Comma)) {
                            self.advance();
                            args.push(self.parse_expression());
                        }
                    }
                    self.expect(Token::RParen);
                    Expr::Call(name, args)
                } else {
                    Expr::Var(name)
                }
            }
            Token::LParen => {
                let inner = self.parse_expression();
                self.expect(Token::RParen);
                inner
            }
            other => panic!("unexpected token in expression: {:?}", other),
        };

        // field access chains: .field or ->field
        while matches!(self.peek(), Some(Token::Dot | Token::Arrow)) {
            self.advance();
            let field = self.expect_ident();
            expr = Expr::FieldAccess(Box::new(expr), field);
        }

        expr
    }
}

pub fn parse(tokens: Vec<Token>) -> Program {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use crate::c_compiler::ast::*;
    use crate::c_compiler::lexer::{tokenize, Token};

    use super::parse;

    // ── Helper ──────────────────────────────────────────────────

    fn parse_src(src: &str) -> Program {
        parse(tokenize(src))
    }

    // ── Example programs ────────────────────────────────────────

    #[test]
    fn test_hello() {
        let prog = parse_src("int main() { print_i32(42); return 0; }");
        assert_eq!(prog.functions.len(), 1);
        let f = &prog.functions[0];
        assert_eq!(f.name, "main");
        assert_eq!(f.return_type, Type::Int);
        assert!(f.params.is_empty());
        assert_eq!(
            f.body,
            vec![
                Stmt::Expr(Expr::Call("print_i32".into(), vec![Expr::IntLit(42)])),
                Stmt::Return(Expr::IntLit(0)),
            ]
        );
    }

    #[test]
    fn test_arithmetic() {
        let src = r#"
            int add(int a, int b) { return a + b; }
            int main() {
                int x = add(10, 20);
                print_i32(x);
                return 0;
            }
        "#;
        let prog = parse_src(src);
        assert_eq!(prog.functions.len(), 2);

        let add = &prog.functions[0];
        assert_eq!(add.name, "add");
        assert_eq!(
            add.params,
            vec![("a".into(), Type::Int), ("b".into(), Type::Int)]
        );
        assert_eq!(
            add.body,
            vec![Stmt::Return(Expr::BinOp(
                Box::new(Expr::Var("a".into())),
                Op::Add,
                Box::new(Expr::Var("b".into())),
            ))]
        );

        let main = &prog.functions[1];
        assert_eq!(main.name, "main");
        assert_eq!(
            main.body[0],
            Stmt::Let(
                "x".into(),
                Type::Int,
                Some(Expr::Call(
                    "add".into(),
                    vec![Expr::IntLit(10), Expr::IntLit(20)]
                ))
            )
        );
    }

    #[test]
    fn test_strings() {
        let src = r#"
            int main() {
                print_str("hello world");
                print_i32(123);
                return 0;
            }
        "#;
        let prog = parse_src(src);
        let main = &prog.functions[0];
        assert_eq!(
            main.body[0],
            Stmt::Expr(Expr::Call(
                "print_str".into(),
                vec![Expr::StringLit("hello world".into())]
            ))
        );
    }

    #[test]
    fn test_structs() {
        let src = r#"
            struct Point { int x; int y; };
            int main() {
                struct Point p;
                p.x = 10;
                p.y = 20;
                print_i32(p.x);
                print_i32(p.y);
                return 0;
            }
        "#;
        let prog = parse_src(src);

        assert_eq!(prog.structs.len(), 1);
        assert_eq!(
            prog.structs[0],
            StructDef {
                name: "Point".into(),
                fields: vec![("x".into(), Type::Int), ("y".into(), Type::Int)],
            }
        );

        let main = &prog.functions[0];
        assert_eq!(main.body[0], Stmt::Let("p".into(), Type::Struct("Point".into()), None));
        assert_eq!(main.body[1], Stmt::FieldAssign("p".into(), "x".into(), Expr::IntLit(10)));
        assert_eq!(main.body[2], Stmt::FieldAssign("p".into(), "y".into(), Expr::IntLit(20)));
        assert_eq!(
            main.body[3],
            Stmt::Expr(Expr::Call(
                "print_i32".into(),
                vec![Expr::FieldAccess(Box::new(Expr::Var("p".into())), "x".into())]
            ))
        );
    }

    // ── Expression precedence ───────────────────────────────────

    #[test]
    fn test_precedence() {
        // a + b * c  →  Add(a, Mul(b, c))
        let prog = parse_src("int f() { return a + b * c; }");
        assert_eq!(
            prog.functions[0].body[0],
            Stmt::Return(Expr::BinOp(
                Box::new(Expr::Var("a".into())),
                Op::Add,
                Box::new(Expr::BinOp(
                    Box::new(Expr::Var("b".into())),
                    Op::Mul,
                    Box::new(Expr::Var("c".into())),
                )),
            ))
        );
    }

    #[test]
    fn test_left_associativity() {
        // a - b - c  →  Sub(Sub(a, b), c)
        let prog = parse_src("int f() { return a - b - c; }");
        assert_eq!(
            prog.functions[0].body[0],
            Stmt::Return(Expr::BinOp(
                Box::new(Expr::BinOp(
                    Box::new(Expr::Var("a".into())),
                    Op::Sub,
                    Box::new(Expr::Var("b".into())),
                )),
                Op::Sub,
                Box::new(Expr::Var("c".into())),
            ))
        );
    }

    #[test]
    fn test_parenthesized_expr() {
        // (a + b) * c  →  Mul(Add(a, b), c)
        let prog = parse_src("int f() { return (a + b) * c; }");
        assert_eq!(
            prog.functions[0].body[0],
            Stmt::Return(Expr::BinOp(
                Box::new(Expr::BinOp(
                    Box::new(Expr::Var("a".into())),
                    Op::Add,
                    Box::new(Expr::Var("b".into())),
                )),
                Op::Mul,
                Box::new(Expr::Var("c".into())),
            ))
        );
    }

    // ── Edge cases ──────────────────────────────────────────────

    #[test]
    fn test_empty_function() {
        let prog = parse_src("int noop() { }");
        assert_eq!(prog.functions[0].body.len(), 0);
    }

    #[test]
    fn test_variable_assign() {
        let prog = parse_src("int f() { int x = 1; x = 2; return x; }");
        assert_eq!(prog.functions[0].body[0], Stmt::Let("x".into(), Type::Int, Some(Expr::IntLit(1))));
        assert_eq!(prog.functions[0].body[1], Stmt::Assign("x".into(), Expr::IntLit(2)));
        assert_eq!(prog.functions[0].body[2], Stmt::Return(Expr::Var("x".into())));
    }

    #[test]
    fn test_nested_calls() {
        let prog = parse_src("int f() { return add(mul(2, 3), 4); }");
        assert_eq!(
            prog.functions[0].body[0],
            Stmt::Return(Expr::Call(
                "add".into(),
                vec![
                    Expr::Call("mul".into(), vec![Expr::IntLit(2), Expr::IntLit(3)]),
                    Expr::IntLit(4),
                ]
            ))
        );
    }

    #[test]
    fn test_arrow_access() {
        let prog = parse_src("int f() { return p->x; }");
        assert_eq!(
            prog.functions[0].body[0],
            Stmt::Return(Expr::FieldAccess(
                Box::new(Expr::Var("p".into())),
                "x".into(),
            ))
        );
    }

    #[test]
    fn test_multiple_structs_and_functions() {
        let src = r#"
            struct A { int val; };
            struct B { float f; };
            int foo() { return 1; }
            int bar() { return 2; }
        "#;
        let prog = parse_src(src);
        assert_eq!(prog.structs.len(), 2);
        assert_eq!(prog.functions.len(), 2);
        assert_eq!(prog.structs[0].name, "A");
        assert_eq!(prog.structs[1].name, "B");
        assert_eq!(prog.functions[0].name, "foo");
        assert_eq!(prog.functions[1].name, "bar");
    }
}
