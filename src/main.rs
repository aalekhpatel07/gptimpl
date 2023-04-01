use std::{path, fs::read_to_string};
use chatimpl::*;
use regex::Regex;

use clap::Parser;
use chatgpt::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref CODE_SNIPPET_RE: Regex = Regex::new(r"(?s)```python(.*)```").unwrap();
}


#[derive(Parser)]
pub struct Opts {
    #[clap(env = "OPENAI_API_KEY")]
    pub apikey: String,
    #[clap(short, long)]
    pub file_path: path::PathBuf
}

pub fn extract_snippet_from_chatgpt_response(content: &str) -> Option<String> {
    CODE_SNIPPET_RE
    .captures(content)
    .map(
        |capture| 
        capture
        .get(1)
        .unwrap()
    )
    .map(|snippet| snippet.as_str().to_owned())
}

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    let file_contents = read_to_string(opts.file_path)?;
    let mut py_source_file = PySourceFile::new(&file_contents);

    py_source_file.parse()?;
    let client = ChatGPT::new(opts.apikey)?;

    let mut conversation = client.new_conversation();

    for defn in &mut py_source_file.functions {
        let message = defn.describe();
        let response = conversation.send_message(message).await?;
        let reply_contents = response.message().content.clone();
        let code_snippet = extract_snippet_from_chatgpt_response(&reply_contents);
        if let Some(snippet) = &code_snippet {
            defn.set_implementation(snippet);
        } else {
            println!("Failed to extract snippet from chatgpt response");
        }
    }

    println!("Updated: {:#?}", py_source_file);

    Ok(())
}
