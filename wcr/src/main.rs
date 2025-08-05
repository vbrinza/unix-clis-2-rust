use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[cfg(test)]
mod tests {
    use super::{FileInfo, count};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the word.\nI just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 2,
            num_words: 10,
            num_chars: 47,
            num_bytes: 47,
        };
        assert_eq!(info.unwrap(), expected);
    }
}

#[derive(Debug, PartialEq)]
struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(short, long)]
    lines: bool,

    #[arg(short, long)]
    words: bool,

    #[arg(short('c'), long)]
    bytes: bool,

    #[arg(short('m'), long, conflicts_with("bytes"))]
    chars: bool,
}

fn count(mut file: impl BufRead) -> Result<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;
    let mut line = String::new();

    loop {
        let line_bytes = file.read_line(&mut line)?;
        if line_bytes == 0 {
            break;
        }
        num_bytes += line_bytes;
        num_lines += 1;
        num_words += line.split_whitespace().count();
        num_chars += line.chars().count();
        line.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{value:>8}")
    } else {
        "".to_string()
    }
}

fn run(mut args: Args) -> Result<()> {
    if [args.words, args.bytes, args.chars, args.lines]
        .iter()
        .all(|v| v == &false)
    {
        args.lines = true;
        args.words = true;
        args.bytes = true;
    }

    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &args.files {
        match open(filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(file) => {
                let info = count(file)?;
                println!(
                    "{}{}{}{}{}",
                    format_field(info.num_lines, args.lines),
                    format_field(info.num_words, args.words),
                    format_field(info.num_bytes, args.bytes),
                    format_field(info.num_chars, args.chars),
                    if filename == "-" {
                        "".to_string()
                    } else {
                        format!(" {filename}")
                    }
                );
                total_lines += info.num_lines;
                total_words += info.num_words;
                total_bytes += info.num_bytes;
                total_chars += info.num_chars;
            }
        }
    }
    if args.files.len() > 1 {
        println!(
            "{}{}{}{} total",
            format_field(total_lines, args.lines),
            format_field(total_words, args.words),
            format_field(total_bytes, args.bytes),
            format_field(total_chars, args.chars)
        )
    }
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
