use anyhow::Result;
use clap::Parser;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

mod test;

/// Count the file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// file path
    file: Option<String>,

    /// count the number of bytes in a file
    #[arg(short, long)]
    count: bool,

    /// count the number of lines in a file
    #[arg(short, long)]
    line: bool,

    /// count the number of words in a file
    #[arg(short, long)]
    word: bool,

    /// count the number of characters in a file
    #[arg(short, long = "character")]
    multibyte: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(ref path) = args.file {
        process_file(&args, path)?;
    } else {
        process_stdin(&args)?;
    }

    Ok(())
}

fn process_file(args: &Args, path: &str) -> Result<()> {
    let path = path.split("/").last().unwrap();

    if args.count {
        let byte_count = byte_count(path)?;
        println!("{:?} {}", byte_count, path);
    }

    if args.line {
        let line_count = line_count(path)?;
        println!("{:?} {}", line_count, path);
    }

    if args.word {
        let word_count = word_count(path)?;
        println!("{:?} {}", word_count, path);
    }

    if args.multibyte {
        let char_count = char_count(path)?;
        println!("{:?} {}", char_count, path);
    }

    if !args.count && !args.line && !args.word && !args.multibyte {
        let byte_count = byte_count(path)?;
        let word_count = word_count(path)?;
        let line_count = line_count(path)?;

        println!("{line_count} {word_count} {byte_count} {path}");
    }

    Ok(())
}

fn process_stdin(args: &Args) -> Result<()> {
    let mut stdin = std::io::stdin();
    // let _ = stdin.lock();
    let mut buf = vec![];
    stdin.read_to_end(&mut buf)?;

    if args.count {
        let byte_count = buf.len();
        println!("{:?} ", byte_count);
    }

    if args.line {
        let content = String::from_utf8(buf.clone())?;
        let line_count = content.split("\n").count();
        println!("{:?} ", line_count);
    }

    if args.word {
        let content = String::from_utf8(buf.clone())?;
        let word_count = content.split_whitespace().count();
        println!("{:?} ", word_count);
    }

    if args.multibyte {
        let content = String::from_utf8(buf.clone())?;
        let char_count = content.chars().count();
        println!("{:?} ", char_count);
    }

    if !args.count && !args.line && !args.word && !args.multibyte {
        let content = String::from_utf8(buf.clone())?;
        let line_count = content.split("\n").count();
        let byte_count = buf.len();
        let word_count = content.split_whitespace().count();
        let char_count = content.chars().count();

        println!("{line_count} {word_count} {byte_count} {char_count}");
    }

    Ok(())
}

fn byte_count(path: &str) -> Result<usize> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    Ok(file.read_to_string(&mut buf)?)
}

fn line_count(path: &str) -> Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(&file);
    Ok(reader.lines().count())
}

fn word_count(path: &str) -> Result<usize> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf.trim().split_whitespace().count())
}

fn char_count(path: &str) -> Result<usize> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf.chars().count())
}
