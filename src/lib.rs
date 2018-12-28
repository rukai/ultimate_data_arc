use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ByteOrder, ReadBytesExt};
use failure::Error;
use scroll::{Pread, LE};

mod parse;
use crate::parse::*;

/// The data.arc file starts with a magic number to identify it as a data.arc
/// It is assumed that any error that occurs on a file starting with the magic number is an internal error
/// i.e. a bug that needs to be fixed.
pub enum ParseError {
    /// The file doesn't start with the magic number 0xabcdef9876543210 so it is not a data.arc file
    NotDataArc,
    /// A bug that needs to be fixed
    InternalError (Error)
}

pub struct DataArc {
    buffer: Vec<u8>,

    // offsets into the buffer taken derived from NodeSection
    bulkfile_hash_lookup: usize,
    bulkfiles_by_name: usize,
    bulkfile_lookup_to_fileidx: usize,
    file_pairs: usize,
    another_hash_table: usize,
    big_hashes: usize,
    big_files: usize,
    folder_hash_lookup: usize,
    trees: usize,
    sub_files1: usize,
    sub_files2: usize,
    folder_to_big_hash: usize,
    file_lookup_buckets: usize,
    file_lookup: usize,
    numbers: usize,

    first_hash_bucket: HashBucket,
}

impl DataArc {
    /// Parse the passed `data.arc` file.
    pub fn new(mut file: File) -> Result<DataArc, ParseError> {
        if let Ok(magic) = file.read_u64::<LittleEndian>() {
            if magic != 0xabcdef9876543210 {
                return Err(ParseError::NotDataArc);
            }
        } else {
            return Err(ParseError::NotDataArc);
        }

        DataArc::internal_new(file).map_err(|err| ParseError::InternalError(err))
    }

    pub fn internal_new(mut file: File) -> Result<DataArc, Error> {
        let mut buffer = vec!(0; ARC_HEADER_SIZE);
        file.read_exact(&mut buffer)?;
        let header: ArcHeader = buffer.pread_with(0, LE)?;

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

            let mut buffer = vec!(0; node_header.file_size as usize - NODE_HEADER_SIZE);
            file.read_exact(&mut buffer)?;
            (node_header, buffer)
        };

        // The node_header tells us how many entries are in each section.
        // From this we know the end of each section and thus the start of the next section.
        let bulkfile_hash_lookup = ENTRY_TRIPLET_SIZE * node_header.movie_count as usize;
        let bulkfiles_by_name = bulkfile_hash_lookup + ENTRY_PAIR_SIZE * node_header.part1_count as usize;
        let bulkfile_lookup_to_fileidx = bulkfiles_by_name + ENTRY_TRIPLET_SIZE * node_header.part1_count as usize;
        let file_pairs = bulkfile_lookup_to_fileidx + 4 * node_header.part2_count as usize;
        let another_hash_table = file_pairs + FILE_PAIR_SIZE * node_header.music_file_count as usize;
        let big_hashes = another_hash_table + ENTRY_TRIPLET_SIZE * node_header.another_hash_table_size as usize;
        let big_files = big_hashes + BIG_HASH_ENTRY_SIZE * node_header.folder_count as usize;
        let folder_hash_lookup = big_files + BIG_FILE_ENTRY_SIZE * (node_header.file_count1 + node_header.file_count2) as usize;
        let trees = folder_hash_lookup + ENTRY_PAIR_SIZE * node_header.hash_folder_count as usize;
        let sub_files1 = trees + TREE_ENTRY_SIZE * node_header.tree_count as usize;
        let sub_files2 = sub_files1 + FILE_ENTRY_SIZE * node_header.sub_files1_count as usize;
        let folder_to_big_hash = sub_files2 + FILE_ENTRY_SIZE * node_header.sub_files2_count as usize;
        let file_lookup_buckets = folder_to_big_hash + ENTRY_PAIR_SIZE * node_header.folder_count as usize;
        let first_hash_bucket: HashBucket = (&buffer[file_lookup_buckets..]).pread_with(0, LE)?;
        let file_lookup = file_lookup_buckets + HASH_BUCKET_SIZE * (first_hash_bucket.num_entries as usize + 1);
        let numbers = file_lookup + ENTRY_PAIR_SIZE * node_header.file_lookup_count as usize;

        Ok(DataArc{
            buffer,

            // offsets into the buffer taken derived from NodeSection
            bulkfile_hash_lookup,
            bulkfiles_by_name,
            bulkfile_lookup_to_fileidx,
            file_pairs,
            another_hash_table,
            big_hashes,
            big_files,
            folder_hash_lookup,
            trees,
            sub_files1,
            sub_files2,
            folder_to_big_hash,
            file_lookup_buckets,
            file_lookup,
            numbers,

            first_hash_bucket,
        })
    }

    pub fn get_file(&self, file_name: &str) -> Result<Vec<u8>, Error> {
        let hash = hash40(file_name);
        let offset = self.file_lookup_buckets + HASH_BUCKET_SIZE * (hash % self.first_hash_bucket.num_entries as u64 + 1) as usize;
        let bucket: HashBucket = self.buffer[offset..].pread_with(0, LE)?;
        println!("{:x}", hash);
        println!("{:x}", bucket.num_entries);
        Ok(vec!())
    }

    pub fn debug_print(&self) -> Result<(), Error> {
        // TODO: print all elements
        println!("bulkfile_category_info: {:x?}", read_triplet(&self.buffer[..]));
        println!("bulkfile_hash_lookup: {:x?}", read_pair(&self.buffer[self.bulkfile_hash_lookup..]));
        println!("bulkfiles_by_name: {:x?}", read_triplet(&self.buffer[self.bulkfiles_by_name..]));
        println!("bulkfile_lookup_tofileidx: {:x?}", LittleEndian::read_u32(&self.buffer[self.bulkfile_lookup_to_fileidx..]));
        let file_pair: FilePair = (&self.buffer[self.file_pairs..]).pread_with(0, LE)?;
        println!("file_pairs: {:x?}", file_pair);
        println!("another_hash_table: {:x?}", read_triplet(&self.buffer[self.another_hash_table..]));
        println!("big_hashes: {:x?}", read_big_hash_entry(&self.buffer[self.big_hashes..]));
        let big_file: BigFileEntry = (&self.buffer[self.big_files..]).pread_with(0, LE)?;
        println!("big_files: {:x?}", big_file);
        println!("folder_hash_lookup: {:x?}", read_pair(&self.buffer[self.folder_hash_lookup..]));
        println!("trees: {:x?}", read_tree_entry(&self.buffer[self.trees..]));
        let file_entry: FileEntry = (&self.buffer[self.sub_files1..]).pread_with(0, LE)?;
        println!("sub_files1: {:x?}", file_entry);
        let file_entry: FileEntry = (&self.buffer[self.sub_files2..]).pread_with(0, LE)?;
        println!("sub_files2: {:x?}", file_entry);
        println!("folder_to_big_hash: {:x?}", read_pair(&self.buffer[self.folder_to_big_hash..]));
        let hash_bucket: HashBucket = (&self.buffer[self.file_lookup_buckets..]).pread_with(0, LE)?;
        println!("file_lookup_buckets: {:x?}", hash_bucket);
        println!("file_lookup: {:x?}", read_pair(&self.buffer[self.file_lookup..]));
        println!("numbers: {:x?}", read_pair(&self.buffer[self.numbers..]));

        Ok(())
    }
}

fn hash40(name: &str) -> u64 {
    crc::crc32::checksum_ieee(name.as_bytes()) as u64 | ((name.len() as u64 & 0xFF) << 32)
}
