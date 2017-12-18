extern crate easy_shortcuts as es;
extern crate lapp;
use es::traits::*;
use std::path::{Path,PathBuf};
use std::{env,process,str,io};

// the one constructable error in stdlib
fn io_error(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

// macro to hide some ugly ifs
macro_rules! assert_err {
    ( $cond:expr , $msg:expr ) => {
        if ! $cond {
            return Err(io_error($msg));
        }
    }
}

// define a custom Lapp type, rust_or_md_file
struct RustFile {
    path: PathBuf
}

impl str::FromStr for RustFile {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let path = PathBuf::from(s);
        assert_err!(path.exists(),"file does not exist");
        {
            let ext = path.extension().ok_or_else(|| io_error("file has no extension"))?;
            assert_err!(ext == "md" || ext == "rs", "extension must be either .rs or .md");
        }
        assert_err!(path.parent().unwrap() == Path::new(""), "must be a plain filename in current directory");
        Ok(RustFile{path: path})
    }
}

const VERSION: &str = "0.1.2";

const USAGE: &str = "
cargo docgen. Compiles and runs doc test snippets.
    These are in the same format as accepted by `cargo test`,
    and are then output in the correct commmented form
    for  pasting in your project. Must be run in some
    subdirectory of a library crate.

    -m, --module module test (//!) (Default is ///)
    -M, --module-doc input is a Markdown file containing
        code examples. Assumes `--module`
    -i, --indent (default '0') indent in spaces ('4') or tabs ('1t')

    -q, --question optional support for ? error handling
    -n, --no-run  compile but don't run

    -V, --version

    <script> (rust_or_md_file) plain filename containing doc test snippet.
    If extension is .md assumes --module-doc, must otherwise
    have extension .rs

    https://github.com/stevedonovan/cargo-docgen/blob/master/readme.md
";

pub struct Config<'a> {
    pub file: PathBuf,
    pub module: bool,
    pub module_doc: bool,
    pub question: bool,
    pub indent: String,
    pub comment: String,
    pub examples: PathBuf,
    pub crate_name: String,
    pub no_run: bool,
    pub args: lapp::Args<'a>,
}

impl <'a> Config<'a> {
    pub fn new() -> Config<'a> {
        let mut args = lapp::Args::new(USAGE).start(2);
        args.user_types(&["rust_or_md_file"]);
        args.parse();

        if args.get_bool("version") {
            println!("version {}",VERSION);
            process::exit(0);
        }

        let (crate_name,examples) = get_crate();
        let mut res = Config {
            file: args.get::<RustFile>("script").path,
            module: args.get_bool("module"),
            module_doc: args.get_bool("module-doc"),
            question: args.get_bool("question"),
            indent: get_indent(args.get_string("indent")),
            comment: "".into(),
            crate_name: crate_name,
            examples: examples,
            no_run: args.get_bool("no-run"),
            args: args,
        };
        res.set_comment();
        res
    }

    fn set_comment(&mut self) {
        if self.file.extension().unwrap() == "md" {
            self.module_doc = true;
        }
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
            let examples = crate_dir.join("examples");
            return (
                crate_name.replace('-',"_").into(), // to make Rust happy...
                examples
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
