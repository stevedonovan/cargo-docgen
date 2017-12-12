pub fn dump_indented(text: &str, comment: &str) {
    let mut out = String::new();
    append_indented(&mut out, text, comment);
    print!("{}", out);
}

pub fn append_indented(dest: &mut String, src: &str, indent: &str) {
    dest.extend(src.lines().map(|s| format!("{} {}\n",indent,s)));
}

pub fn findstr(haystack: &str, needle: &str) -> Option<(usize,usize)> {
    if let Some(pos) = haystack.find(needle) {
        Some((pos,pos+needle.len()))
    } else {
        None
    }
}
