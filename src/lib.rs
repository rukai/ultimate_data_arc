use byteorder::{LittleEndian, ReadBytesExt};

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use scroll::{Pread, LE};
use scroll_derive::Pread;
use failure::Error;

/// The data.arc file starts with a magic number to identify it as a data.arc
/// It is assumed that any error that occurs on a file starting with the magic number is an internal error
/// i.e. a bug that needs to be fixed.
pub enum ParseError {
    /// The file doesn't start with the magic number 0xabcdef9876543210 so it is not a data.arc file
    NotDataArc,
    /// A bug that needs to be fixed
    InternalError (Error)
}

pub fn parse(mut file: File) -> Result<(), ParseError> {
    if let Ok(magic) = file.read_u64::<LittleEndian>() {
        if magic != 0xabcdef9876543210 {
            return Err(ParseError::NotDataArc);
        }
    } else {
        return Err(ParseError::NotDataArc);
    }

    internal_parse(file).map_err(|err| ParseError::InternalError(err))
}

#[derive(Debug, Pread)]
struct ArcHeader {
    music_file_section_offset: u64,
    file_section_offset: u64,
    music_section_offset: u64,
    node_section_offset: u64,
    unk_section_offset: u64,
}

#[derive(Debug, Pread)]
struct CompressedNodeHeader {
    data_start: u32,
    decomp_size: u32,
    comp_size: u32,
    zstd_comp_size: u32,
}

#[derive(Debug, Pread)]
struct NodeHeader {
    file_size: u32,
    folder_count: u32,
    file_count1: u32,
    file_name_count: u32,

    sub_file_count1: u32,
    last_table_count: u32,
    hash_folder_count: u32,
    file_information_count: u32,

    file_count2: u32,
    sub_file_count2: u32,
    unk1: u32,
    unk2: u32,
    another_hash_table_size: u8,
    unk3: u8,
    unk4: u16,

    movie_count: u32,
    part1_count: u32,
    part2_count: u32,
    music_file_count: u32,
}

pub fn internal_parse(mut file: File) -> Result<(), Error> {
    let mut buffer = vec!(0; 0x28);
    file.read_exact(&mut buffer)?;
    let header: ArcHeader = buffer.pread_with(0, LE)?;
    println!("{:x?}", header);

    file.seek(SeekFrom::Start(header.node_section_offset))?;

    let mut buffer = vec!(0; 0x20);
    file.read_exact(&mut buffer)?;
    let compressed: CompressedNodeHeader = buffer.pread_with(0, LE)?;

    if compressed.data_start < 0x100 {
        // TODO: Handle compressed node
        unimplemented!()
    } else {
        file.seek(SeekFrom::Start(header.node_section_offset))?;
        let mut buffer = vec!(0; 0x1000);
        file.read_exact(&mut buffer)?;
        let node_header: NodeHeader = buffer.pread_with(0, LE)?;
        println!("{:x?}", node_header);
        hexdump::hexdump(&buffer);
    }
    
    Ok(())
}
