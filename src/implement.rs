use std::collections::HashMap;

use rustpython_parser::ast::*;


#[derive(Debug, Clone, Default)]
pub struct PyArgument {
    pub r#type: Option<String>,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct PyFunction {
    pub name: String,
    pub args: Vec<PyArgument>,
    pub doc_string: Option<Located<String>>,
    pub location: Location,
    pub end_location: Option<Location>
}


pub fn get_args(args: &Box<Arguments>) -> Vec<PyArgument> {

    args
    .args
    .iter()
    .filter_map(|arg| {
        let ArgData { arg, annotation, .. } = &arg.node;

        let name = arg.clone();
        // FIXME: Make sure this returns n.
        let Some(ann) = annotation.as_ref() else {
            return None;
        };
        let ExprKind::Name { id, .. } = &ann.node else {
            return None;
        };

        
        // TODO: Handle default args.
        Some( PyArgument {
            name,
            r#type: Some(id.clone())
        })
    })
    .into_iter()
    .collect()
}

pub fn get_docstring(stmt: &Located<StmtKind>) -> Option<Located<String>> {

    let StmtKind::Expr { value } = &stmt.node else {
        return None;
    };

    let start_location = &stmt.location;
    let end_location = &stmt.end_location.clone();

    let ExprKind::Constant { value, .. } = &value.node else {
        return None;
    };

    let Constant::Str(doc_string) = value else {
        return None;
    };

    let literal_value = doc_string.trim().to_string();
    Some( Located {
        location: start_location.clone(),
        end_location: end_location.clone(),
        custom: (),
        node: literal_value
    })
}

pub fn parse_def(source: &str) -> Result<Vec<PyFunction>, Box<dyn std::error::Error>> {
    let suite = rustpython_parser::parser::parse_program(&source, "<test>")?;

    Ok(
    suite
    .into_iter()
    .filter_map(|stmt| {
        match &stmt.node {
            StmtKind::FunctionDef { name, args, body, decorator_list, returns, type_comment } => {

                let mut py_function = PyFunction::default();
                py_function.name = name.clone();
                py_function.location = stmt.location.clone();
                py_function.end_location = stmt.end_location.clone();
                // First statement might be the docstring,
                // Ignore all else.
                if let Some(body_stmt) = body.first() {
                    py_function.doc_string = get_docstring(&body_stmt);
                }

                py_function.args = get_args(args);
                Some(py_function)
            },
            _ => {
                None
            }
        }
    })
    .collect())
}


#[cfg(test)]
mod tests {
    use rustpython_parser::parser::parse_program;

    use super::*;

    const SOURCE_FIB: &str = r#"
def fibonacci(n: int, a: float) -> int:
    """
    Return the n-th fibonacci number.
    """
"#;

    #[test]
    fn test_parse() {
        let suite = parse_program(&SOURCE_FIB, "<test>");
        assert!(suite.is_ok());
    }

    #[test]
    fn test_implement_function() {
        let stuff = parse_def(&SOURCE_FIB).unwrap();
        println!("{:#?}", stuff);
        println!("{}", SOURCE_FIB);
    }
}