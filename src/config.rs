extern crate easy_shortcuts as es;
extern crate lapp;
use es::traits::*;
use std::path::{Path,PathBuf};
use std::env;


pub const VERSION: &str = "0.1.1";

const USAGE: &str = "
cargo docgen. Compiles and runs a test snippet
    -m, --module module test (//!)
    -M, --module-doc input is a Markdown file containing
        code examples. Assumes `--module`
    -q, --question optional support for ? error handling
    -i, --indent (default '0') indent in spaces ('4') or tabs ('1t')
    -n, --no-run
    -V, --version
    <script> (string) file containing doc test snippet
";

pub struct Config {
    pub file: String,
    pub module: bool,
    pub module_doc: bool,
    pub question: bool,
    pub indent: String,
    pub comment: String,
    pub examples: PathBuf,
    pub crate_name: String,
    pub no_run: bool,
    pub version: bool,
}

impl Config {
    pub fn new() -> Config {
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
            version: args.get_bool("version"),
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

// Very hacky stuff - we want the ACTUAL crate name, not the project name
// So look just past [package] and scrape the name...
// (borrowed from runner crate)
fn toml_crate_name(cargo_toml: &Path) -> String {
    let name_line = es::lines(es::open(cargo_toml))
        .skip_while(|line| line.trim() != "[package]")
        .skip(1)
        .skip_while(|line| ! line.starts_with("name "))
        .next().or_die("totally fked Cargo.toml");
    let idx = name_line.find('"').or_die("no name?");
    (&name_line[(idx+1)..(name_line.len()-1)]).into()

}

fn get_crate() -> (String,PathBuf) {
    let mut crate_dir = env::current_dir().or_die("cannot get current directory");
    loop {
        let cargo_toml = crate_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let crate_name = toml_crate_name(&cargo_toml);
            return (
                crate_name.replace('-',"_").into(),
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
