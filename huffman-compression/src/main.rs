use anyhow::Result;
use clap::Parser;
use huffman::HuffmanCompression;

pub mod huffman;

#[cfg(test)]
mod test;

/// Count the file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// input file path
    #[arg(short, long)]
    input: String,

    /// output file path
    #[arg(short, long)]
    output: String,

    /// option for encode file
    #[arg(short, long)]
    encode: bool,

    /// option for decode compressed file
    #[arg(short, long)]
    decode: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let huffman = HuffmanCompression {
        src: args.input,
        dst: args.output,
    };

    if args.encode {
        huffman.encode()?;
    }

    if args.decode {
        huffman.decode()?;
    }

    Ok(())
}
