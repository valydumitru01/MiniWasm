#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i32),
    Float(f64),
    String(String),
    Identifier(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Dot,

    // Assignment
    Equal,

    // Arrow (->)
    Arrow,

    // Keywords
    KwInt,
    KwFloat,
    KwReturn,
    KwStruct,
    KwConst,
    KwChar,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }

            // Slash or comment
            '/' => {
                chars.next();
                match chars.peek() {
                    // Line comment
                    Some(&'/') => {
                        chars.next();
                        while let Some(&c) = chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            chars.next();
                        }
                    }
                    // Block comment
                    Some(&'*') => {
                        chars.next();
                        loop {
                            match chars.next() {
                                Some('*') => {
                                    if chars.peek() == Some(&'/') {
                                        chars.next();
                                        break;
                                    }
                                }
                                Some(_) => {}
                                None => break, // unterminated block comment
                            }
                        }
                    }
                    // Division operator
                    _ => tokens.push(Token::Slash),
                }
            }

            // Minus or Arrow (->)
            '-' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                } else {
                    tokens.push(Token::Minus);
                }
            }

            // Single-character tokens
            '+' => { chars.next(); tokens.push(Token::Plus); }
            '*' => { chars.next(); tokens.push(Token::Star); }
            '(' => { chars.next(); tokens.push(Token::LParen); }
            ')' => { chars.next(); tokens.push(Token::RParen); }
            '{' => { chars.next(); tokens.push(Token::LBrace); }
            '}' => { chars.next(); tokens.push(Token::RBrace); }
            ';' => { chars.next(); tokens.push(Token::Semicolon); }
            ',' => { chars.next(); tokens.push(Token::Comma); }
            '.' => { chars.next(); tokens.push(Token::Dot); }
            '=' => { chars.next(); tokens.push(Token::Equal); }

            // String literals
            '"' => {
                chars.next(); // consume opening quote
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('\\') => {
                            // Escape sequences
                            match chars.next() {
                                Some('n') => s.push('\n'),
                                Some('t') => s.push('\t'),
                                Some('\\') => s.push('\\'),
                                Some('"') => s.push('"'),
                                Some('0') => s.push('\0'),
                                Some(c) => {
                                    s.push('\\');
                                    s.push(c);
                                }
                                None => break,
                            }
                        }
                        Some('"') => break,
                        Some(c) => s.push(c),
                        None => break, // unterminated string
                    }
                }
                tokens.push(Token::String(s));
            }

            // Number literals
            '0'..='9' => {
                let mut num_str = String::new();
                let mut is_float = false;
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        num_str.push(c);
                        chars.next();
                    } else if c == '.' && !is_float {
                        is_float = true;
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if is_float {
                    tokens.push(Token::Float(num_str.parse::<f64>().unwrap()));
                } else {
                    tokens.push(Token::Int(num_str.parse::<i32>().unwrap()));
                }
            }

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let token = match ident.as_str() {
                    "int" => Token::KwInt,
                    "float" => Token::KwFloat,
                    "return" => Token::KwReturn,
                    "struct" => Token::KwStruct,
                    "const" => Token::KwConst,
                    "char" => Token::KwChar,
                    _ => Token::Identifier(ident),
                };
                tokens.push(token);
            }

            other => {
                panic!("unexpected character: '{}'", other);
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        let tokens = tokenize("int main() { print_i32(42); return 0; }");
        assert_eq!(tokens, vec![
            Token::KwInt,
            Token::Identifier("main".into()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::Identifier("print_i32".into()),
            Token::LParen,
            Token::Int(42),
            Token::RParen,
            Token::Semicolon,
            Token::KwReturn,
            Token::Int(0),
            Token::Semicolon,
            Token::RBrace,
        ]);
    }

    #[test]
    fn test_arithmetic() {
        let tokens = tokenize("int add(int a, int b) { return a + b; }");
        assert_eq!(tokens, vec![
            Token::KwInt,
            Token::Identifier("add".into()),
            Token::LParen,
            Token::KwInt,
            Token::Identifier("a".into()),
            Token::Comma,
            Token::KwInt,
            Token::Identifier("b".into()),
            Token::RParen,
            Token::LBrace,
            Token::KwReturn,
            Token::Identifier("a".into()),
            Token::Plus,
            Token::Identifier("b".into()),
            Token::Semicolon,
            Token::RBrace,
        ]);
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize(r#"print_str("hello world");"#);
        assert_eq!(tokens, vec![
            Token::Identifier("print_str".into()),
            Token::LParen,
            Token::String("hello world".into()),
            Token::RParen,
            Token::Semicolon,
        ]);
    }

    #[test]
    fn test_string_escape_sequences() {
        let tokens = tokenize(r#""hello\nworld\t!""#);
        assert_eq!(tokens, vec![
            Token::String("hello\nworld\t!".into()),
        ]);
    }

    #[test]
    fn test_struct_def() {
        let tokens = tokenize("struct Point { int x; int y; };");
        assert_eq!(tokens, vec![
            Token::KwStruct,
            Token::Identifier("Point".into()),
            Token::LBrace,
            Token::KwInt,
            Token::Identifier("x".into()),
            Token::Semicolon,
            Token::KwInt,
            Token::Identifier("y".into()),
            Token::Semicolon,
            Token::RBrace,
            Token::Semicolon,
        ]);
    }

    #[test]
    fn test_dot_access() {
        let tokens = tokenize("p.x = 10;");
        assert_eq!(tokens, vec![
            Token::Identifier("p".into()),
            Token::Dot,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Int(10),
            Token::Semicolon,
        ]);
    }

    #[test]
    fn test_arrow() {
        let tokens = tokenize("p->x");
        assert_eq!(tokens, vec![
            Token::Identifier("p".into()),
            Token::Arrow,
            Token::Identifier("x".into()),
        ]);
    }

    #[test]
    fn test_float_literal() {
        let tokens = tokenize("3.14");
        assert_eq!(tokens, vec![Token::Float(3.14)]);
    }

    #[test]
    fn test_line_comment() {
        let tokens = tokenize("int x; // this is a comment\nint y;");
        assert_eq!(tokens, vec![
            Token::KwInt,
            Token::Identifier("x".into()),
            Token::Semicolon,
            Token::KwInt,
            Token::Identifier("y".into()),
            Token::Semicolon,
        ]);
    }

    #[test]
    fn test_block_comment() {
        let tokens = tokenize("int /* skip this */ x;");
        assert_eq!(tokens, vec![
            Token::KwInt,
            Token::Identifier("x".into()),
            Token::Semicolon,
        ]);
    }

    #[test]
    fn test_division() {
        let tokens = tokenize("10 / 2");
        assert_eq!(tokens, vec![
            Token::Int(10),
            Token::Slash,
            Token::Int(2),
        ]);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize(""), vec![]);
    }

    #[test]
    fn test_all_keywords() {
        let tokens = tokenize("int float return struct const char");
        assert_eq!(tokens, vec![
            Token::KwInt,
            Token::KwFloat,
            Token::KwReturn,
            Token::KwStruct,
            Token::KwConst,
            Token::KwChar,
        ]);
    }

    #[test]
    fn test_full_program() {
        let src = r#"
            // A simple program
            int main() {
                print_str("hello world");
                print_i32(123);
                return 0;
            }
        "#;
        let tokens = tokenize(src);
        assert_eq!(tokens, vec![
            Token::KwInt,
            Token::Identifier("main".into()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::Identifier("print_str".into()),
            Token::LParen,
            Token::String("hello world".into()),
            Token::RParen,
            Token::Semicolon,
            Token::Identifier("print_i32".into()),
            Token::LParen,
            Token::Int(123),
            Token::RParen,
            Token::Semicolon,
            Token::KwReturn,
            Token::Int(0),
            Token::Semicolon,
            Token::RBrace,
        ]);
    }
}
