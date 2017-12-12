//! A simple doc test runner. Packages up snippets using similar rules
//! to cargo test.
//! It is ASSUMED that you are in an immediate subdirectory of the crate
//! root

extern crate easy_shortcuts as es;
extern crate lapp;
use es::traits::*;

mod cache;
mod example;
mod config;
mod util;

use example::Example;
use util::{findstr,dump_indented};


fn main() {
    let mut config = config::Config::new();

    if config.version {
        println!("version {}",config::VERSION);
        return;
    }

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


