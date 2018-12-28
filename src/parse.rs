use byteorder::{LittleEndian, ByteOrder};
use scroll_derive::Pread;

#[derive(Debug, Pread)]
pub(crate) struct ArcHeader {
    pub music_file_section_offset: u64,
    pub file_section_offset: u64,
    pub music_section_offset: u64,
    pub node_section_offset: u64,
    pub unk_section_offset: u64,
}
pub(crate) const ARC_HEADER_SIZE: usize = 0x28;

#[derive(Debug, Pread)]
pub(crate) struct CompressedNodeHeader {
    pub data_start: u32,
    pub decomp_size: u32,
    pub comp_size: u32,
    pub zstd_comp_size: u32,
}
pub(crate) const COMPRESSED_NODE_HEADER_SIZE: usize = 0x10;

#[derive(Debug, Pread)]
pub(crate) struct NodeHeader {
    pub file_size: u32,
    pub folder_count: u32,
    pub file_count1: u32,
    pub tree_count: u32,

    pub sub_files1_count: u32,
    pub file_lookup_count: u32,
    pub hash_folder_count: u32,
    pub file_information_count: u32,

    pub file_count2: u32,
    pub sub_files2_count: u32,
    pub unk1: u32,
    pub unk2: u32,

    pub another_hash_table_size: u8,
    pub unk3: u8,
    pub unk4: u16,

    pub movie_count: u32,
    pub part1_count: u32,
    pub part2_count: u32,
    pub music_file_count: u32,
}
pub(crate) const NODE_HEADER_SIZE: usize = 0x44;

#[derive(Debug)]
pub(crate) struct EntryTriplet {
    pub hash: u64, // 0x28 bits
    pub meta: u32, // 0x18 bits
    pub meta2: u32,
}
pub(crate) const ENTRY_TRIPLET_SIZE: usize = 0xc;

pub(crate) fn read_triplet(data: &[u8]) -> EntryTriplet {
    let hash = LittleEndian::read_u64(&[data[0], data[1], data[2], data[3], data[4], 0, 0, 0]);
    let meta = LittleEndian::read_u32(&[data[5], data[6], data[7], 0]);
    let meta2 = LittleEndian::read_u32(&data[0x8..]);
    EntryTriplet { hash, meta, meta2 }
}

#[derive(Debug)]
pub(crate) struct EntryPair {
    pub hash: u64, // 0x28 bits
    pub meta: u32, // 0x18 bits
}
pub(crate) const ENTRY_PAIR_SIZE: usize = 0x8;

pub(crate) fn read_pair(data: &[u8]) -> EntryPair {
    let hash = LittleEndian::read_u64(&[data[0], data[1], data[2], data[3], data[4], 0, 0, 0]);
    let meta = LittleEndian::read_u32(&[data[5], data[6], data[7], 0]);
    EntryPair { hash, meta }
}

#[derive(Debug)]
pub(crate) struct BigHashEntry {
    pub path: EntryPair,
    pub folder: EntryPair,
    pub parent: EntryPair,
    pub hash4: EntryPair,
    pub suboffset_start: u32,
    pub num_files: u32,
    pub unk3: u32,
    pub unk4: u16,
    pub unk5: u16,
    pub unk6: u8,
    pub unk7: u8,
    pub unk8: u8,
    pub unk9: u8,
}
pub(crate) const BIG_HASH_ENTRY_SIZE: usize = 0x34;

pub(crate) fn read_big_hash_entry(data: &[u8]) -> BigHashEntry {
    BigHashEntry {
        path: read_pair(&data[0x00..]),
        folder: read_pair(&data[0x08..]),
        parent: read_pair(&data[0x10..]),
        hash4: read_pair(&data[0x18..]),
        suboffset_start: LittleEndian::read_u32(&data[0x20..]),
        num_files: LittleEndian::read_u32(&data[0x24..]),
        unk3: LittleEndian::read_u32(&data[0x28..]),
        unk4: LittleEndian::read_u16(&data[0x2c..]),
        unk5: LittleEndian::read_u16(&data[0x2e..]),
        unk6: data[0x30],
        unk7: data[0x31],
        unk8: data[0x32],
        unk9: data[0x33],
    }
}

#[derive(Debug)]
pub(crate) struct TreeEntry {
    pub path: EntryPair,
    pub ext: EntryPair,
    pub folder: EntryPair,
    pub file: EntryPair,
    pub suboffset_index: u32,
    pub flags: u32,
}
pub(crate) const TREE_ENTRY_SIZE: usize = 0x28;

pub(crate) fn read_tree_entry(data: &[u8]) -> TreeEntry {
    TreeEntry {
        path: read_pair(&data[0x00..]),
        ext: read_pair(&data[0x08..]),
        folder: read_pair(&data[0x10..]),
        file: read_pair(&data[0x18..]),
        suboffset_index: LittleEndian::read_u32(&data[0x20..]),
        flags: LittleEndian::read_u32(&data[0x24..]),
    }
}

#[derive(Debug, Pread)]
pub(crate) struct FilePair {
    pub size: u64,
    pub offset: u64,
}
pub(crate) const FILE_PAIR_SIZE: usize = 0x10;

#[derive(Debug, Pread)]
pub(crate) struct BigFileEntry {
    pub offset: u64,
    pub decomp_size: u32,
    pub comp_size: u32,
    pub suboffset_index: u32,
    pub files: u32,
    pub unk3: u32,
}
pub(crate) const BIG_FILE_ENTRY_SIZE: usize = 0x1c;

#[derive(Debug, Pread)]
pub(crate) struct FileEntry {
    pub offset: u32,
    pub comp_size: u32,
    pub decomp_size: u32,
    pub flags: u32,
}
pub(crate) const FILE_ENTRY_SIZE: usize = 0x10;

#[derive(Debug, Pread)]
pub(crate) struct HashBucket {
    pub index: u32,
    pub num_entries: u32,
}
pub(crate) const HASH_BUCKET_SIZE: usize = 0x08;
