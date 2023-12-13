#[derive(Eq, PartialEq, Debug)]
pub enum QueryPart {
    Const(String),
    Var(String),
}

#[derive(Eq, PartialEq)]
enum QueryLexerMode {
    Const,
    VarStart,
    VarName,
}

pub struct QueryLexer {
    buf: String,
    mode: QueryLexerMode,
    parts: Vec<QueryPart>,
}

impl QueryLexer {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            mode: QueryLexerMode::Const,
            parts: vec![],
        }
    }

    pub fn consume(mut self, str: &str) -> QueryLexer {
        for c in str.chars() {
            self.step(c);
        }
        self
    }

    pub fn step(&mut self, c: char) {
        if c == '$' && self.mode == QueryLexerMode::Const {
            if !self.buf.is_empty() {
                self.parts.push(QueryPart::Const(self.buf.clone()));
                self.buf = "".to_string();
            }
            self.mode = QueryLexerMode::VarStart;
        } else if c == '{' && self.mode == QueryLexerMode::VarStart {
            self.mode = QueryLexerMode::VarName;
        } else if c == '}' && self.mode == QueryLexerMode::VarName {
            self.mode = QueryLexerMode::Const;
            if !self.buf.is_empty() {
                self.parts.push(QueryPart::Var(self.buf.clone()));
                self.buf = "".to_string();
            }
        } else {
            self.buf.push(c);
        }
    }

    pub fn compile(mut self) -> Vec<QueryPart> {
        if !self.buf.is_empty() {
            if self.mode == QueryLexerMode::Const {
                self.parts.push(QueryPart::Const(self.buf.clone()));
            } else if self.mode == QueryLexerMode::VarName {
                self.parts.push(QueryPart::Var(self.buf.clone()));
            }
        }
        self.parts
    }
}

#[cfg(test)]
mod test {
    use crate::query::lexer::{QueryLexer, QueryPart};

    #[test]
    fn should_parse_const() {
        let parts = QueryLexer::new().consume("test").compile();
        assert_eq!(1, parts.len());
        assert_eq!(QueryPart::Const("test".to_string()), parts[0]);
    }

    #[test]
    fn should_parse_var() {
        let parts = QueryLexer::new().consume("test ${asd} test").compile();
        assert_eq!(3, parts.len());
        assert_eq!(QueryPart::Const("test ".to_string()), parts[0]);
        assert_eq!(QueryPart::Var("asd".to_string()), parts[1]);
        assert_eq!(QueryPart::Const(" test".to_string()), parts[2]);
    }
}
