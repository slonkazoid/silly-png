use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process::exit,
};

use clap::Parser;
use png::text_metadata::{EncodableTextChunk, TEXtChunk};

// Do not change
const PNG_HEADER_SIZE: usize = 8;
const IHDR_CHUNK_SIZE: usize = 13 + 12;
const IHDR_END_IDX: usize = IHDR_CHUNK_SIZE + PNG_HEADER_SIZE;

fn format_code(
    offsets: &[u64],
    block_sizes: &[u64],
    counts: &[u64],
    code: &str,
    digits: usize,
) -> String {
    let [offsets, block_sizes, counts] = [offsets, block_sizes, counts].map(|x| {
        x.iter()
            .map(|y| format!("{:0>1$}", y, digits))
            .collect::<Vec<String>>()
            .join(" ")
    });

    format!(
        "
# shellscript embedded with silly-png
# https://gitlab.com/slonkazoid/silly-png
# https://slonk.ing/
_offsets=({})
_block_sizes=({})
_counts=({})
# usage: extract [<index> [-p]]
extract() {{
    local i=${{1:-0}}
    local status=none
    [[ \"$2\" == '-p' ]] && status=progress
    dd if=\"$0\" \\
        skip=${{_offsets[$i]:?file $i not found}} \\
        bs=${{_block_sizes[$i]}} \\
        count=${{_counts[$i]}} \\
        status=$status
}}

# script start
({})
# script end

exit
",
        offsets, block_sizes, counts, code
    )
}

#[derive(Parser, Debug)]
#[command(author, version, about = "silly little png shell script embedder")]
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

    #[arg(
        short,
        long,
        default_value_t = 4096 * 1024,
        help = "Maximum block size",
    )]
    max_block_size: u64,

    #[arg(
        short,
        long,
        default_value_t = 16,
        help = "Number of digits in the generated array entries"
    )]
    digits: usize,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Force build even if it may not be interpretable"
    )]
    force: bool,

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

fn evil_file_size(file: &mut File) -> Result<u64, std::io::Error> {
    let len = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(0))?;

    Ok(len)
}

fn main() {
    let args = Args::parse();

    let mut input_file = File::open(&args.png).unwrap();

    let mut header: [u8; IHDR_END_IDX] = [0; IHDR_END_IDX];
    input_file.read_exact(&mut header).unwrap();

    if [
        0x7d, // }
        0x7b, // {
        0x29, // )
        0x28, // (
        0x27, // '
        0x22, // "
              // TODO: add more
    ]
    .iter()
    .any(|x| header.contains(x))
    {
        let fatal = !args.force;
        eprintln!(
            "{}: the png header might contain a syntax error",
            if fatal { "fatal" } else { "warning" }
        );
        eprintln!(
            "try playing around with the image parameters (like width, height) to mitigate this"
        );
       
        if fatal {
            exit(1);
        }
    }

    let mut script_file = File::open(&args.script).unwrap();
    let len = script_file.seek(SeekFrom::End(0)).unwrap();
    script_file.seek(SeekFrom::Start(0)).unwrap();

    let mut script = String::with_capacity(len as usize);
    script_file.read_to_string(&mut script).unwrap();

    let file_count = args.files.as_ref().map(|x| x.len()).unwrap_or(0);

    // set up placeholder for length counting
    let placeholder_code = format_code(
        &[0].repeat(file_count),
        &[0].repeat(file_count),
        &[0].repeat(file_count),
        &script,
        args.digits,
    );
    let placeholder_text_chunk = TEXtChunk::new(args.keyword.clone(), placeholder_code);
    let mut encoded_placeholder = Vec::new();
    placeholder_text_chunk
        .encode(&mut encoded_placeholder)
        .unwrap();
    let placeholder_len = encoded_placeholder.len();
    drop(placeholder_text_chunk);

    let mut output_file = File::create(args.output.unwrap_or_else(|| {
        let mut path = args.png.clone();
        path.set_extension("silly.png");
        path
    }))
    .unwrap();

    eprintln!("writing png data");
    output_file.write_all(&header).unwrap();
    output_file
        .seek(SeekFrom::Current(placeholder_len as i64))
        .unwrap();
    std::io::copy(&mut input_file, &mut output_file).unwrap();

    let (offsets, block_sizes, counts) = if let Some(files) = args.files {
        let mut offsets = Vec::with_capacity(file_count);
        let mut block_sizes = Vec::with_capacity(file_count);
        let mut counts = Vec::with_capacity(file_count);

        let mut last = output_file.stream_position().unwrap();

        for path in files {
            eprintln!("copying {} to png file", path.display());
            let mut file = File::open(path).expect("couldn't open file");
            let size = evil_file_size(&mut file).unwrap();

            let block_size = if size > args.max_block_size {
                divisors::get_divisors(size)
                    .into_iter()
                    .filter(|x| *x <= args.max_block_size)
                    .collect::<Vec<_>>()
                    .pop()
                    .unwrap_or(1)
            } else {
                size
            };

            let padding = last.next_multiple_of(block_size) - last;
            output_file
                .write_all(&[0].repeat(padding as usize))
                .unwrap();

            let written = padding + std::io::copy(&mut file, &mut output_file).unwrap();
            offsets.push((padding + last) / block_size);
            block_sizes.push(block_size);
            counts.push(size / block_size);

            last += written;
        }

        (offsets, block_sizes, counts)
    } else {
        Default::default()
    };

    eprintln!("writing shellscript");
    let code = format_code(&offsets, &block_sizes, &counts, &script, args.digits);
    let text_chunk = TEXtChunk::new(args.keyword, code);
    output_file
        .seek(SeekFrom::Start(IHDR_END_IDX as u64))
        .unwrap();
    text_chunk.encode(&mut output_file).unwrap();
}
