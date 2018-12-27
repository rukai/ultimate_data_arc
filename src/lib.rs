use byteorder::{LittleEndian, ByteOrder, ReadBytesExt};

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

/// Parse the passed `data.arc` file.
/// TODO: The idea is to return a type that allows exploring the filesystem
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
const ARC_HEADER_SIZE: usize = 0x28;

#[derive(Debug, Pread)]
struct CompressedNodeHeader {
    data_start: u32,
    decomp_size: u32,
    comp_size: u32,
    zstd_comp_size: u32,
}
const COMPRESSED_NODE_HEADER_SIZE: usize = 0x10;

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
const NODE_HEADER_SIZE: usize = 0x44;

#[derive(Debug)]
struct EntryTriplet {
    hash: u64, // 0x28 bits
    meta: u32, // 0x18 bits
    meta2: u32,
}
const ENTRY_TRIPLET_SIZE: usize = 0xc;

fn read_triplet(data: &[u8]) -> EntryTriplet {
    let hash = LittleEndian::read_u64(&[data[0], data[1], data[2], data[3], data[4], 0, 0, 0]);
    let meta = LittleEndian::read_u32(&[data[5], data[6], data[7], 0]);
    let meta2 = LittleEndian::read_u32(&data[0x8..]);
    EntryTriplet { hash, meta, meta2 }
}

#[derive(Debug)]
struct EntryPair {
    hash: u64, // 0x28 bits
    meta: u32, // 0x18 bits
}
const ENTRY_PAIR_SIZE: usize = 0x8;

fn read_pair(data: &[u8]) -> EntryPair {
    let hash = LittleEndian::read_u64(&[data[0], data[1], data[2], data[3], data[4], 0, 0, 0]);
    let meta = LittleEndian::read_u32(&[data[5], data[6], data[7], 0]);
    EntryPair { hash, meta }
}

#[derive(Debug)]
struct BigHashEntry {
    path: EntryPair,
    folder: EntryPair,
    parent: EntryPair,
    hash4: EntryPair,
    suboffset_start: u32,
    num_files: u32,
    unk3: u32,
    unk4: u16,
    unk5: u16,
    unk6: u8,
    unk7: u8,
    unk8: u8,
    unk9: u8,
}
const BIG_HASH_ENTRY_SIZE: usize = 0x30;

fn read_big_hash_entry(data: &[u8]) -> BigHashEntry {
    BigHashEntry {
        path: read_pair(&data[0x00..]),
        folder: read_pair(&data[0x08..]),
        parent: read_pair(&data[0x10..]),
        hash4: read_pair(&data[0x18..]),
        suboffset_start: LittleEndian::read_u32(&data[0x1c..]),
        num_files: LittleEndian::read_u32(&data[0x20..]),
        unk3: LittleEndian::read_u32(&data[0x24..]),
        unk4: LittleEndian::read_u16(&data[0x28..]),
        unk5: LittleEndian::read_u16(&data[0x2A..]),
        unk6: data[0x2C],
        unk7: data[0x2D],
        unk8: data[0x2E],
        unk9: data[0x2F],
    }
}

#[derive(Debug, Pread)]
struct FilePair {
    size: u64,
    offset: u64,
}
const FILE_PAIR_SIZE: usize = 0x10;

#[derive(Debug, Pread)]
struct BigFileEntry {
    offset: u64,
    decomp_size: u32,
    comp_size: u32,
    suboffset_index: u32,
    files: u32,
    unk3: u32,
}
const BIG_FILE_ENTRY_SIZE: usize = 0x1c;

pub fn internal_parse(mut file: File) -> Result<(), Error> {
    let mut buffer = vec!(0; ARC_HEADER_SIZE);
    file.read_exact(&mut buffer)?;
    let header: ArcHeader = buffer.pread_with(0, LE)?;
    println!("{:x?}", header);

    file.seek(SeekFrom::Start(header.node_section_offset))?;

    let mut buffer = vec!(0; COMPRESSED_NODE_HEADER_SIZE);
    file.read_exact(&mut buffer)?;
    let compressed: CompressedNodeHeader = buffer.pread_with(0, LE)?;

    let (node_header, buffer) = if compressed.data_start < 0x100 {
        // TODO: Handle compressed node
        unimplemented!()
    } else {
        file.seek(SeekFrom::Start(header.node_section_offset))?;
        let mut buffer = vec!(0; NODE_HEADER_SIZE);
        file.read_exact(&mut buffer)?;
        let node_header: NodeHeader = buffer.pread_with(0, LE)?;
        println!("{:x?}", node_header);

        let mut buffer = vec!(0; node_header.file_size as usize - NODE_HEADER_SIZE);
        file.read_exact(&mut buffer)?;
        (node_header, buffer)
    };

    // The node_header tells us how many entries are in each section.
    // From this we know the end of each section and thus the start of the next section.
    let bulkfile_category_info = &buffer[..];
    let bulkfile_hash_lookup = &buffer[ENTRY_TRIPLET_SIZE * node_header.movie_count as usize..];
    let bulk_files_by_name = &bulkfile_hash_lookup[ENTRY_PAIR_SIZE * node_header.part1_count as usize..];
    let bulkfile_lookup_to_fileidx = &bulk_files_by_name[ENTRY_TRIPLET_SIZE * node_header.part1_count as usize..];
    let file_pairs = &bulkfile_lookup_to_fileidx[4 * node_header.part2_count as usize..];
    let another_hash_table = &file_pairs[FILE_PAIR_SIZE * node_header.music_file_count as usize..];
    let big_hashes = &another_hash_table[ENTRY_TRIPLET_SIZE * node_header.another_hash_table_size as usize..];
    let big_files = &big_hashes[BIG_HASH_ENTRY_SIZE * node_header.folder_count as usize..];

    // Debug prints
    // TODO: print all elements
    // TODO: Log instead of print
    hexdump::hexdump(&bulkfile_category_info[..1000]);
    println!("bulkfile_category_info: {:x?}", read_triplet(bulkfile_category_info));
    println!("bulkfile_hash_lookup: {:x?}", read_pair(bulkfile_hash_lookup));
    println!("bulk_files_by_name: {:x?}", read_triplet(bulk_files_by_name));
    println!("bulkfile_lookup_tofileidx: {:x?}", LittleEndian::read_u32(&bulkfile_lookup_to_fileidx));
    let file_pair: FilePair = file_pairs.pread_with(0, LE)?;
    println!("file_pairs: {:x?}", file_pair);
    println!("another_hash_table: {:x?}", read_triplet(another_hash_table));
    println!("big_hashes: {:x?}", read_big_hash_entry(big_hashes));
    let big_file: BigFileEntry = big_files.pread_with(0, LE)?;
    println!("big_files: {:x?}", big_file);

    Ok(())
}
