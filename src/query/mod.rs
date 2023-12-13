use crate::error::{Error, Result};
use crate::query::lexer::{QueryLexer, QueryPart};
use supabase_wrappers::interface::Value;
use supabase_wrappers::prelude::Qual;
use pgrx::prelude::*;

mod lexer;

fn format_str(string: String) -> String {
    if string.starts_with("'") && string.ends_with("'") {
        string[1..(string.len() - 1)].to_string()
    } else {
        string
    }
}

pub struct PromQuery {
    parts: Vec<QueryPart>,
}

impl PromQuery {
    pub fn parse(str: &str) -> PromQuery {
        let parts = QueryLexer::new().consume(str).compile();
        PromQuery { parts }
    }

    pub fn compile(&self, quals: &[Qual]) -> Result<String> {
        let mut req = String::new();
        for part in &self.parts {
            match part {
                QueryPart::Const(c) => req.push_str(c),
                QueryPart::Var(name) => {
                    let var = quals
                        .iter()
                        .filter(|q| q.operator == "in" || q.operator == "=")
                        .find(|q| &q.field == name)
                        .ok_or_else(|| Error::VariableNotFound(name.to_string()))?;
                    match &var.value {
                        Value::Cell(v) => req.push_str(&format_str(v.to_string())),
                        Value::Array(v) => req.push_str(
                            &v.iter()
                                .map(|c| c.to_string())
                                .map(|s| format_str(s))
                                .collect::<Vec<String>>()
                                .join("|"),
                        ),
                    }
                }
            }
        }
        Ok(req)
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use crate::query::PromQuery;
    use supabase_wrappers::prelude::{Qual, Cell, Value};
    use pgrx::prelude::*;

    #[pg_test]
    fn should_compile_simple() {
        let out = PromQuery::parse("test").compile(&[]).unwrap();
        assert_eq!("test", out);
    }

    #[pg_test]
    fn should_compile_vars() {
        let out = PromQuery::parse("test ${a} ${b} asd")
            .compile(&[
                Qual {
                    value: Value::Array(vec![
                        Cell::String("qwe".to_string()),
                        Cell::String("asd".to_string()),
                    ]),
                    operator: "in".to_string(),
                    field: "a".to_string(),
                    param: None,
                    use_or: false,
                },
                Qual {
                    value: Value::Cell(Cell::String("bvb".to_string())),
                    operator: "=".to_string(),
                    field: "b".to_string(),
                    param: None,
                    use_or: false,
                },
            ])
            .unwrap();
        assert_eq!("test qwe|asd bvb asd", out);
    }
}
