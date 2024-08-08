use anyhow::{Ok, Result};
use clap::Parser;
use std::{
    fs::File,
    io::{self, stdin, stdout, BufRead, BufReader, Write},
};

#[cfg(test)]
mod test;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Specifies the field to cut
    #[arg(short, long, value_parser=parse_filed)]
    field: usize,

    /// The input file
    file: Option<String>,

    /// Specified the delimiter to use, default is tab character
    #[arg(short, long, default_value = "\t")]
    delimiter: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(file) = args.file {
        process_file(&file, &args.delimiter, args.field)?;
    } else {
        process_stdin(&args.delimiter, args.field)?;
    }

    Ok(())
}

fn parse_filed(field: &str) -> std::result::Result<usize, String> {
    use std::result::Result::Ok;

    match field.parse::<usize>() {
        Ok(num) if num >= 1 => Ok(num),
        Ok(_) => Err(String::from(
            "The field value must be greater than or equal to 1",
        )),
        Err(_) => Err(String::from(
            "The field value must be a valid positive integer",
        )),
    }
}

fn process_file(file: &str, delimiter: &str, field: usize) -> Result<()> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let tokens = line.split(delimiter).collect::<Vec<_>>();

        if let Err(e) = stdout().write_all(format!("{}\n", tokens[field - 1]).as_bytes()) {
            if e.kind() == io::ErrorKind::BrokenPipe {
                break;
            }
        }
    }

    Ok(())
}

fn process_stdin(delimiter: &str, field: usize) -> Result<()> {
    for line in stdin().lines() {
        let line = line?;
        let tokens = line.split(delimiter).collect::<Vec<_>>();
        println!("{}", tokens[field - 1]);
    }

    Ok(())
}
