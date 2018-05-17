use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;

#[macro_use]
extern crate structopt;
use structopt::StructOpt;

extern crate base64;

extern crate crc;
use crc::{Hasher32, crc32};

const IEND: [u8; 12] = [0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82];

// convert u32 to array of bytes in network byte order
// this is a perfectly valid use of unsafe! honestly there should be functions that do this in
// the std library because the operation (u32-> 4 u8) is entirely safe.
fn u32_bytes_be(n: u32) -> [u8;4] {
    return unsafe {std::mem::transmute(n.to_be())}
}

#[derive(Debug, Clone, StructOpt)]
enum PCArgs {
    #[structopt(name="encode")]
    /// Encode a file into a png.
    Encode {
        #[structopt(parse(from_os_str))]
        /// The name of the png file to hide the input file in.
        input_png: PathBuf,
        #[structopt(parse(from_os_str))]
        /// The name of the file to hide inside the input PNG.
        input_file: PathBuf,
        #[structopt(short="o", long="output", parse(from_os_str))]
        /// The name of the new png file containing the hidden input data.
        output: PathBuf,
    },
    #[structopt(name="decode")]
    /// Decode a file hidden in a png.
    Decode {
        #[structopt(parse(from_os_str))]
        /// The name of the png file containing the hidden data.
        input_png: PathBuf,
        #[structopt(short="o", long="output", parse(from_os_str))]
        /// The name of the file to write the hidden data into.
        output: PathBuf,
    }
}

fn encode_text_chunk(data: &[u8]) -> Vec<u8> {
    const max_length: usize = (2<<31) - 1; // as defined in png spec

    // TODO could break file into multiple chunks
    if data.len() > max_length {
        panic!("Input data is too big to store in png chunk! Aborting.");
    }

    let encoded_data = base64::encode(data);
    if encoded_data.len() > max_length+5 { // +5 for the "data" keyword plus null byte
        panic!("Base64 encoded data is too big to store in png! Aborting.")
    }
    let chunk_length: u32 = (encoded_data.len() + 5) as u32; // cast is valid because we checked it's less than 2^31.

    let mut output = Vec::new();
    output.extend(u32_bytes_be(chunk_length).iter()); 
    output.extend("tEXt".as_bytes()); // chunk type tag
    output.extend("data\0".as_bytes()); // keyword and then null byte
    output.extend(encoded_data.as_bytes());
    // TODO this could be made faster by precalculating and preallocating the size of the base64
    // encoded data (basically multiply by 8/6 plus some rounding) and then using
    // base64::encode_config_slice

    let mut crc_hasher = crc32::Digest::new(crc32::IEEE);
    crc_hasher.write(&output[4..]); // skip length field when calculating crc
    // however, if the length is incorrect, the checksums will not match because they will be run
    // over different bytes.

    let crc = crc_hasher.sum32();
    output.extend(&u32_bytes_be(crc));

    return output;
}

fn png_without_iend(data: &[u8]) -> &[u8] {
    let size = data.len()-12; // 4 for length, 4 for IEND, 4 for CRC
    assert!(data[size..] == IEND);
    &data[..size]
}

fn read_bytes(path: PathBuf) -> Vec<u8> {
    let mut buf = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();
    buf
}

fn encode_file(input_png: PathBuf, input_file: PathBuf, output: PathBuf) {
    let png_bytes = read_bytes(input_png);
    let input_bytes = read_bytes(input_file);
    let mut output_bytes = Vec::new();
    let mut output_file = File::create(output).unwrap();

    output_bytes.extend(png_without_iend(&png_bytes));
    output_bytes.extend(encode_text_chunk(&input_bytes));
    output_bytes.extend(&IEND);

    output_file.write(&output_bytes).unwrap();
}

fn decode_file(input_png: PathBuf, output: PathBuf) {
    
}

fn main() {
    let args = PCArgs::from_args();

    match args {
        PCArgs::Encode{input_png, input_file, output} => encode_file(input_png, input_file, output),
        PCArgs::Decode{input_png, output} => decode_file(input_png, output),
    }
}
