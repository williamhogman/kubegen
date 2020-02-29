use snailquote::unescape;
use std::str;

#[derive(Debug)]
pub enum EnvRow {
    Blank,
    Env(String, String),
    Comment(String),
}

fn split_once(in_string: &str, sep: char) -> Option<(&str, &str)> {
    let mut splitter = in_string.splitn(2, sep);
    splitter
        .next()
        .and_then(|first| splitter.next().map(|second| (first, second)))
}

pub fn parse_line(entry: &[u8]) -> Option<EnvRow> {
    str::from_utf8(entry).ok().and_then(|l| {
        let line = l.trim();
        if line.starts_with('#') {
            Some(EnvRow::Comment(line.trim_start_matches('#').to_owned()))
        } else if line.len() == 0 {
            Some(EnvRow::Blank)
        } else {
            split_once(line, '=').and_then(|(key, val)| {
                unescape(val)
                    .ok()
                    .map(|parsed_val| EnvRow::Env(key.to_owned(), parsed_val.to_owned()))
            })
        }
    })
}