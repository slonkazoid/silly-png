use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use clap::Parser;
use png::text_metadata::{EncodableTextChunk, TEXtChunk};

const PNG_HEADER_SIZE: usize = 8;
const FUCK: usize = 12;
const IHDR_SIZE: usize = 13;
const IHDR_END_IDX: usize = IHDR_SIZE + FUCK + PNG_HEADER_SIZE;

macro_rules! format_code {
    ($($arg:tt)*) => {
        format!("
# shellscript embedded with silly-png
# https://gitlab.com/slonkazoid/silly-png
# https://slonk.ing/
offsets=({})
lengths=({})
extract() {{
    dd if=$0 skip=${{offsets[${{1:-0}}]}}B count=${{lengths[${{1:-0}}]}}B bs=4M status=none
}}

{}
exit
", $($arg)*)
    };
}

#[derive(Parser, Debug)]
#[command(author = "slonkazoid", version = env!("CARGO_PKG_VERSION"), about = "silly little png shell script embedder", long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Output file",
        long_help = "Output file. By default, it will write to 'filename.silly.png'"
    )]
    output: Option<PathBuf>,

    #[arg(
        short,
        long,
        default_value = "sourcecode",
        help = "Keyword for the source code text chunk"
    )]
    keyword: String,

    // TODO
    //#[arg(short, long, default_value = "4M", help = "block size for alignment and such")]
    //block_size: usize,
    #[arg(index = 1, help = "PNG file to operate on")]
    png: PathBuf,

    #[arg(
        index = 2,
        help = "Shellscript to embed",
        long_help = "Shellscript to embed. This shellscript will be embedded in a text chunk after the PNG header"
    )]
    script: PathBuf,

    #[arg(index = 3)]
    files: Option<Vec<PathBuf>>,
}

fn main() {
    let args = Args::parse();

    let mut input_file = File::open(&args.png).unwrap();

    let mut script_file = File::open(&args.script).unwrap();
    let len = script_file.seek(SeekFrom::End(0)).unwrap();
    script_file.seek(SeekFrom::Start(0)).unwrap();

    let mut script = String::with_capacity(len as usize);
    script_file.read_to_string(&mut script).unwrap();

    let offsets_len = 19 * args.files.clone().map(|x| x.len()).unwrap_or(0)
        - if args.files.is_some() { 1 } else { 0 };

    let code = format_code!(" ".repeat(offsets_len), " ".repeat(offsets_len), &script);

    let text_chunk = TEXtChunk::new(args.keyword.clone(), code);
    let mut encoded_text = Vec::new();
    text_chunk.encode(&mut encoded_text).unwrap();

    let mut output_file = File::create(&args.output.unwrap_or_else(|| {
        let mut path = args.png.clone();
        path.set_extension("silly.png");
        path
    }))
    .unwrap();

    eprintln!("writing png data");
    let marker = std::io::copy(
        &mut (&input_file).take(IHDR_END_IDX as u64),
        &mut output_file,
    )
    .unwrap();
    output_file
        .seek(SeekFrom::Current(encoded_text.len() as i64))
        .unwrap();
    std::io::copy(&mut input_file, &mut output_file).unwrap();

    let (offsets, lengths) = if let Some(files) = args.files {
        let mut offsets = String::with_capacity(offsets_len + 1);
        let mut blocks = String::with_capacity(offsets_len + 1);

        let mut last = output_file.seek(SeekFrom::Current(0)).unwrap();
        for file in files {
            eprintln!("copying {} to file", file.display());
            let written = std::io::copy(&mut File::open(&file).unwrap(), &mut output_file).unwrap();
            offsets.push_str(&format!("{:0>18} ", last));
            blocks.push_str(&format!("{:0>18} ", written));
            last += written;
        }

        offsets.pop().unwrap();
        blocks.pop().unwrap();

        (offsets, blocks)
    } else {
        ("".into(), "".into())
    };

    eprintln!("writing shellscript");
    let code = format_code!(&offsets, &lengths, &script);
    let text_chunk = TEXtChunk::new(args.keyword, code);
    output_file.seek(SeekFrom::Start(marker)).unwrap();
    text_chunk.encode(&mut output_file).unwrap();
}
