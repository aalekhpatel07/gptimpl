use std::path::{self, Path, PathBuf};

use rustpython_parser::ast::*;
use askama::Template;

#[derive(Debug, Clone, Default)]
pub struct PyArgument {
    pub r#type: Option<String>,
    pub name: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PySourceFile {
    pub src: String,
    pub functions: Vec<PyFunction>
}

impl PySourceFile {
    pub fn new(source: &str) -> Self {
        Self {
            src: source.into(),
            functions: vec![]
        }
    }

    pub fn parse(&mut self) -> core::result::Result<(), Box<dyn std::error::Error>> {
        self.functions = parse_defs(&self.src)?;
        Ok(())
    }


}


impl PyArgument {
    pub fn describe(&self) -> String {
        let mut s = format!("`{}`", self.name);
        if let Some(r#type) = &self.r#type {
            s.push_str(&format!(" that is of type `{}`", r#type));
        }
        if let Some(default_value) = &self.default_value {
            s.push_str(&format!(" with a default value of `{}`,", default_value));
        }
        s
    }
}


#[derive(Debug, Default, Clone)]
pub struct PyFunction {
    pub name: String,
    pub args: Vec<PyArgument>,
    pub doc_string: Option<Located<String>>,
    pub return_type: Option<String>,
    pub location: Location,
    pub end_location: Option<Location>
}


#[derive(Template, Default)]
#[template(path = "impl.html")]
pub struct ImplTemplate {
    function_name: String,
    has_arguments: bool,
    has_return_type: bool,
    has_docstring: bool,
    argument_block: String,
    return_type: String,
    docstring: String
}



impl PyFunction {

    pub fn as_snippet(&self) -> String {
        let mut result = format!("def {}", self.name);
        result.push_str("(");

        let last_arg_index = self.args.len() - 1;

        for (index, arg) in self.args.iter().enumerate() {
            result.push_str(&format!("{}", arg.name));
            if let Some(r#type) = &arg.r#type {
                result.push_str(&format!(": {}", r#type));
            }
            if let Some(default_value) = &arg.default_value {
                result.push_str(&format!(" = {}", default_value));
            }
            if index != last_arg_index {
                result.push_str(", ");
            }
        }
        result.push_str(")");

        if let Some(return_type) = &self.return_type {
            result.push_str(&format!(" -> {}", return_type));
        }
        result.push_str(":\n");

        if let Some(doc_string) = &self.doc_string {
            result.push_str("    \"\"\"\n    ");
            result.push_str(&doc_string.node);
            result.push_str("\n    \"\"\"");
        }

        result
    }

    pub fn describe(&self) -> String {

        let arguments: Vec<_> = 
            self
            .args
            .iter()
            .map(|arg| arg.describe())
            .collect();

        let argument_block: String = format!("- {}", arguments.join("\n- "));
        
        let template = ImplTemplate {
            function_name: self.name.clone(),
            has_arguments: !self.args.is_empty(),
            has_return_type: self.return_type.is_some(),
            has_docstring: self.doc_string.is_some(),
            argument_block,
            return_type: self.return_type.clone().unwrap_or("".into()),
            docstring: self.doc_string.as_ref().map(|doc_string| doc_string.node.clone()).unwrap_or("".into())
        };

        template.render().unwrap()

    }

    pub fn set_implementation(&mut self, body: &str) -> core::result::Result<(), Box<dyn std::error::Error>> {
        let suite = rustpython_parser::parser::parse_program(&body, "<test>")?;
        
        Ok(())
    }
}

pub trait AsString {
    fn as_string(&self) -> String;
}

impl AsString for Constant {
    fn as_string(&self) -> String {
        match self {
            Constant::None => "None".into(),
            Constant::Bool(b) => {
                if *b {
                    "True".to_owned()
                } else {
                    "False".to_owned()
                }
            },
            Constant::Bytes(bytes) => {
                format!("{:?}", bytes)
            },
            Constant::Complex { real, imag } => {
                format!("{} + {}i", *real, *imag)
            },
            Constant::Ellipsis => {
                "...".into()
            },
            Constant::Float(f) => {
                format!("{}", f)
            },
            Constant::Int(int) => {
                format!("{}", int)
            },
            Constant::Str(s) => {
                s.clone()
            },
            Constant::Tuple(t) => {
                format!("({})", t.iter().map(Self::as_string).collect::<Vec<_>>().join(", "))
            }
        }
    }
}


fn get_args(args: &Box<Arguments>) -> Vec<PyArgument> {

    let mut usual_args: Vec<_> = 
    args
    .args
    .iter()
    .filter_map(|arg| {
        let ArgData { arg, annotation, .. } = &arg.node;

        let name = arg.clone();
        let Some(ann) = annotation.as_ref() else {
            return None;
        };

        let ExprKind::Name { id, .. } = &ann.node else {
            dbg!(&ann.node);
            return None;
        };

        Some( PyArgument {
            name,
            r#type: Some(id.clone()),
            default_value: None
        })
    })
    .into_iter()
    .collect();

    let usual_args_len = usual_args.len();

    args
    .defaults
    .iter()
    .rev()
    .enumerate()
    .for_each(|(index, arg)| {

        let mut py_arg = usual_args.get_mut(usual_args_len - index - 1).unwrap();

        match &arg.node {
            ExprKind::Name { id, .. } => {
                py_arg.default_value = Some(id.clone());
            },
            ExprKind::Constant { value, .. } => {
                py_arg.default_value = Some(value.as_string());
            }
            _ => {}
        }
    });

    usual_args
}

fn get_docstring(stmt: &Located<StmtKind>) -> Option<Located<String>> {

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


pub fn parse_def(old_defn: &mut PyFunction, new_source: &str) {
    let Ok(suite) = rustpython_parser::parser::parse_program(&new_source, "<test>") else {
        return;
    };

    let Some(stmt) = suite.first() else {
        return;
    };

    let StmtKind::FunctionDef { name, args, body, returns, .. } = &stmt.node else {
        return;
    };

    let mut has_docstring: bool = false;
    // First statement might be the docstring,
    // Ignore all else.
    if let Some(body_stmt) = body.first() {
        has_docstring = get_docstring(&body_stmt).is_some();
    }

    if has_docstring {
        // We found a docstring in the new source, so just return the function as is.
        // expr as the function body.
        // Get the start location of the first statement and the end location of the last statement.
        let fn_body = body
        .iter()
        .skip(1)
        .next();


    } else {

        _ = body
        .iter()
        .map(|b| {
            println!("{:#?}", b.node);
        }).collect::<Vec<_>>();
    }

}

pub fn parse_defs(source: &str) -> Result<Vec<PyFunction>, Box<dyn std::error::Error>> {
    let suite = rustpython_parser::parser::parse_program(&source, "<test>")?;

    Ok(
    suite
    .into_iter()
    .filter_map(|stmt| {
        let StmtKind::FunctionDef { name, args, body, returns, .. } = &stmt.node else {
            return None;
        };

        let mut py_function = PyFunction::default();
        py_function.name = name.clone();
        py_function.location = stmt.location.clone();
        py_function.end_location = stmt.end_location.clone();
        
        if let Some(returns) = returns {
            if let ExprKind::Name { id, .. } = &returns.node {
                py_function.return_type = Some(id.clone());
            }
        }
        // First statement might be the docstring,
        // Ignore all else.
        if let Some(body_stmt) = body.first() {
            py_function.doc_string = get_docstring(&body_stmt);
        }

        py_function.args = get_args(args);
        Some(py_function)
    })
    .collect())
}


#[cfg(test)]
mod tests {
    use rustpython_parser::parser::parse_program;

    use super::*;

    const SOURCE_FIB: &str = r#"
def fibonacci(n: int, a: float = 1.0, b: float = None) -> int:
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
        let stuff = parse_defs(&SOURCE_FIB).unwrap();
        println!("{:#?}", stuff);
        println!("{}", SOURCE_FIB);
    }

    #[test]
    fn test_describe_single() {

        const SOURCE: &str = r#"
def fibonacci(n: int, a: float = 1.0, b: float = None) -> int:
    """
    Return the n-th fibonacci number.
    """
"#;

        let expected = r#"Can you implement exactly one function for me in Python whose name is `fibonacci` that takes in the arguments specified below and returns the type specified below. This function should do exactly as the docstring says which is also described below.

Arguments:
- `n` that is of type `int`
- `a` that is of type `float` with a default value of `1`,
- `b` that is of type `float` with a default value of `None`,


Return type:
- `int`

Docstring:
"Return the n-th fibonacci number."

Can you please only return the function implementation and not return any other text other than that? (Not even the "Sure, here's the implementation")"#;

    let definitions = parse_defs(&SOURCE).unwrap();
    let fib = definitions.first().unwrap();
    let observed = fib.describe();
    assert_eq!(expected, observed);
    }

    #[test]
    fn test_describe() {
        const SOURCE: &str = r#"
import typing
def fibonacci(n: int, x: float = None, t: float = ("1", "2")) -> int:
    """
    Return the (n+2)-th fibonacci number.
    foobar
    """
"#;
    let definitions = parse_defs(&SOURCE).unwrap();
    let descriptions = definitions.iter().map(|defn| defn.describe()).collect::<Vec<_>>();
    println!("{:#?}", descriptions);
    println!("{}", definitions.first().unwrap().as_snippet());
    }

    #[test]
    fn test_function_body_injection() {
        let s = "\ndef fibonacci(n: int) -> int:\n    if n <= 1:\n        return n\n    else:\n        return fibonacci(n-1) + fibonacci(n-2)\n";
        // let foo = parse_def(s).unwrap();
        println!("{:#?}", s);
    }
}