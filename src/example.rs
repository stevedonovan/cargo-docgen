use super::config::Config;
use super::es;
use es::traits::*;

use std::fs;
use std::process;

use util::append_indented;

pub struct Example<'a,'b> where 'b: 'a {
    config: &'a Config<'b>,
    code: String,
    example: String,
    before: String,
    after: String,
}

const ALLOW: &[&str] = &["unused_variables", "unused_assignments", "unused_mut",
    "unused_attributes", "dead_code", "unreachable_code"];

impl <'a,'b> Example<'a,'b> {
    // Convert a doc test into a crate example; at minimum, needs
    // a crate reference and a main function. The question mark operator
    // requires a `run` function that returns any error. We will
    // return the full example, plus any before and after.
    // see https://doc.rust-lang.org/book/first-edition/documentation.html#documentation-as-tests

    pub fn new (config: &'a Config<'b>, code: &str) -> Example<'a,'b> {
        let mut template = String::new();
        let mut before = String::new();
        let mut after = String::new();

        // crate-level attributes have to be top!
        let mut iter = code.lines();
        let mut code = String::new();
        while let Some(line) = iter.next() {
            if line.starts_with("#![") {
                template += line;
                template.push('\n');
            } else {
                code += line;
                code.push('\n');
            }
        }

        for lint in ALLOW.iter() {
            template += &format!("#![allow({})]\n",lint);
        }

        // Tests assume 'extern crate your_crate' unless there's already a declaration
        if code.find("extern crate").is_none() {
            template += &format!("extern crate {};\n",config.crate_name);
        }

        // they will also wrap your code in a main function
        if ! config.question {
            template += "fn main() {\n";
            template += &code;
            template += "}\n";
        } else {
            // unless you want to use the question-mark operator;
            // then we have to make up both a run() and main()
            before += "fn run() -> std::result::Result<(),Box<std::error::Error>> {\n";
            template += &before;
            template += &code;
            after += "Ok(())\n}\n\nfn main() {\n   run().unwrap();\n}";
            template += &after;
        }
        println!("template {}",template);

        Example{
            config: config,
            code: code.into(),
            example: template,
            before: before,
            after: after,
        }
    }

    // Present the original snippet with the correct indentation and comments.
    // If there were before and after, these are written out as hidden lines.
    pub fn format(&self) -> String {
        let comment = self.config.comment.as_str();
        let attrib = if self.config.no_run {"rust,no_run"} else {""};
        let start_guard = format!("{} ```{}\n", comment, attrib);
        let end_guard = format!("{} ```\n", comment);
        let hide = format!("{} #", comment);
        let mut snippet = String::new();
        snippet.push_str(&start_guard);
        // before and after neeed '#' so they don't appear in the docs!
        append_indented(&mut snippet,&self.before,&hide);
        append_indented(&mut snippet,&self.code,comment);
        append_indented(&mut snippet,&self.after,&hide);
        snippet.push_str(&end_guard);
        snippet
    }

    // Run the example by copying it to the project examples folder
    // and invoking 'cargo run --example' on it. We return success and
    // additionally any captured stdout
    pub fn run(&self) -> (bool,String) {
        if self.example.len() == 0 {
            self.config.args.quit("please call massage() before run()");
        }
        if ! self.config.examples.is_dir() {
            fs::create_dir(&self.config.examples).or_die("cannot create examples directory");
        }
        let test_file = self.config.examples.join(&self.config.file);
        es::write_all(&test_file,&self.example);

        // let cargo run the example
        let output = process::Command::new("cargo")
            .arg(if self.config.no_run {"build"} else {"run"})
            .arg("-q")
            .arg("--color").arg("always") // let the Colours flow through, man
            .arg("--example")
            .arg(self.config.file.with_extension(""))
            .output().or_die("could not run cargo");

        let errors = String::from_utf8_lossy(&output.stderr);
        if errors.len() > 0 {
            eprintln!("{}", errors);
        }
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        fs::remove_file(&test_file).or_die("can't remove temporary file in examples");
        (output.status.success(), stdout)
    }

}


