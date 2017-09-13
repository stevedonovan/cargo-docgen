//! A simple doc test runner. Packages up snippets using similar rules
//! to cargo test.
//! It is ASSUMED that you are in an immediate subdirectory of the crate
//! root

extern crate easy_shortcuts as es;
extern crate lapp;
use es::traits::*;
use std::path::PathBuf;
use std::env;
use std::fs;
use std::process;

mod cache;

const USAGE: &str = "
cargo docgen. Compiles and runs a test snippet
    -m, --module module test (//!)
    -M, --module-doc input is a Markdown file containing
        code examples. Assumes `--module`
    -q, --question optional support for ? error handling
    -i, --indent (default '0') indent in spaces ('4') or tabs ('1t')
    -n, --no-run
    <script> (string) file containing doc test snippet
";

struct Config {
    file: String,
    module: bool,
    module_doc: bool,
    question: bool,
    indent: String,
    comment: String,
    examples: PathBuf,
    crate_name: String,
    no_run: bool,
}

impl Config {
    fn new() -> Config {
        let mut args = lapp::Args::new(USAGE).start(2);
        args.parse();
        let (crate_name,examples) = get_crate();
        let mut res = Config {
            file: args.get_string("script"),
            module: args.get_bool("module"),
            module_doc: args.get_bool("module-doc"),
            question: args.get_bool("question"),
            indent: get_indent(args.get_string("indent")),
            comment: "".into(),
            crate_name: crate_name,
            examples: examples,
            no_run: args.get_bool("no-run"),
        };
        res.set_comment();
        res
    }

    fn set_comment(&mut self) {
        if self.module_doc {
            self.module = true;
        }
        self.comment = format!("{}//{}",
            if self.module {""} else {&self.indent},
            if self.module {'!'} else {'/'}
        );

    }
}

fn main() {
    let mut config = Config::new();

    let code = es::read_to_string(&config.file);

    if ! config.module_doc {
        // just a doc test snippet
        let ex = Example::new(&config,&code);
        let (ok,stdout) = ex.run();
        if ok {
            if stdout.len() > 0 {
                eprintln!("****** tests will ignore this output ****");
                eprintln!("{}\n******",stdout);
            }
            let snip = ex.format();
            eprintln!("****** Copy and paste this into your code ******\n");
            println!("{}",snip);
        }

    } else {
        // a whole markdown file
        let start_guard = "```rust";
        let guard = "```\n";
        let doc_file = config.file.clone();
        let mut s = code.as_str();
        let mut kount = 1;
        let sep = "---\n";
        let mut snippet_cache = cache::read(&doc_file,sep).or_die("bad cache");
        let comment = config.comment.as_str();
        while let Some((start1,start2)) = findstr(s,start_guard) {
            // just comment the text
            dump_indented(&s[0..start1], comment);
            // and skip the guard and find the end
            s = &s[start2..];
            // ```rust? means the snippet has a Question
            config.question = if s.starts_with('?') {
                s = &s[2..]; // skip ?\n
                true
            } else {
                s = &s[1..]; // skip \n
                false
            };
            let (end1,end2) = findstr(s,guard).or_die("expecting end of code ```");
            let snippet = &s[0..end1];
            config.file = format!("t{}.rs",kount);
            let ex = Example::new(&config,snippet);
            // don't run the test again if we have seen this snippet!
            if snippet_cache.iter().position(|s| s == snippet).is_none() {
                snippet_cache.push(snippet.into());
                let (_,stdout) = ex.run();
                if stdout.len() > 0 {
                    let comment_comment = format!("{} // ",comment);
                    dump_indented(&stdout, &comment_comment);
                }
            }
            print!("{}", ex.format());
            kount += 1;

            // let's look for next code block
            s = &s[end2..];
        }
        dump_indented(s, comment);
        cache::write(&doc_file,snippet_cache,sep).or_die("cannot write cache");
    }

}

struct Example<'a> {
    config: &'a Config,
    code: String,
    example: String,
    before: String,
    after: String,
}


impl <'a> Example<'a> {
    // Convert a doc test into a crate example; at minimum, needs
    // a crate reference and a main function. The question mark operator
    // requires a `run` function that returns any error. We will
    // return the full example, plus any before and after.

    fn new (config: &'a Config, code: &str) -> Example<'a> {
        let mut template = String::new();
        let mut before = String::new();
        let mut after = String::new();

        // Tests assume 'extern crate your_crate' unless there's already a declaration
        if code.find("extern crate").is_none() {
            template += &format!("extern crate {};\n",config.crate_name);
        }

        // they will also wrap your code in a main function
        if ! config.question {
            template += "fn main() {\n";
            template += code;
            template += "}\n";
        } else {
            // unless you want to use the question-mark operator;
            // then we have to make up both a run() and main()
            before += "use std::error::Error;\n\n";
            before += "fn run() -> Result<(),Box<Error>> {\n";
            template += &before;
            template += code;
            after += "Ok(())\n}\n\nfn main() {\n   run().unwrap();\n}";
            template += &after;
        }

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
    fn format(&self) -> String {
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
    fn run(&self) -> (bool,String) {
        if self.example.len() == 0 {
            es::quit("please call massage() before run()");
        }
        if ! fs::metadata(&self.config.examples).is_dir() {
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
            .arg(self.config.file.replace(".rs",""))
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

fn get_crate() -> (String,PathBuf) {
    let mut crate_dir = env::current_dir().or_die("cannot get current directory");
    loop {
        if crate_dir.join("Cargo.toml").exists() {
            let crate_name = crate_dir.file_name().or_die("can't get crate name");
            return (
                crate_name.to_str().unwrap().replace('-',"_").to_string(),
                crate_dir.join("examples")
            );
        }
        if ! crate_dir.pop() {
            break;
        }
    }
    es::quit("not a subdirectory of a Cargo project");

}

fn get_indent(s: String) -> String {
    let (num,postfix) = if s.ends_with("t") {
        s.split_at(s.len()-1)
    } else {
        (s.as_str(),"")
    };
    let ch = if postfix == "t" { '\t' } else { ' ' };
    let spaces: u32 = num.parse().or_die("indent");
    (0..spaces).map(|_| ch).collect()
}

fn findstr(haystack: &str, needle: &str) -> Option<(usize,usize)> {
    if let Some(pos) = haystack.find(needle) {
        Some((pos,pos+needle.len()))
    } else {
        None
    }
}

fn dump_indented(text: &str, comment: &str) {
    let mut out = String::new();
    append_indented(&mut out, text, comment);
    print!("{}", out);
}

fn append_indented(dest: &mut String, src: &str, indent: &str) {
    dest.extend(src.lines().map(|s| format!("{} {}\n",indent,s)));
}
