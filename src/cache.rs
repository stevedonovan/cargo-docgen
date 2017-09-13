use std::fs;
use std::io::prelude::*;
use std::io;

pub fn read(doc: &str, sep: &str) -> io::Result<Vec<String>> {
    let mut res = Vec::new();
    let file = format!("{}.cache",doc);
    if let Ok(mut f) = fs::File::open(&file) {
        let mut text = String::new();
        f.read_to_string(&mut text)?;
        res = text.split(sep).map(|s| s.to_string()).collect();
        // drop the empty string at the end
        res.pop();
    }
    Ok(res)
}

pub fn write(doc: &str, cache: Vec<String>, sep: &str) -> io::Result<()>{
    let file = format!("{}.cache",doc);
    let mut f = fs::File::create(&file)?;
    for s in cache.into_iter() {
        write!(f,"{}",s)?;
        write!(f,"{}",sep)?;
    }
    Ok(())
}
