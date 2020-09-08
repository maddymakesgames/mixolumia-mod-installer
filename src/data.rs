use std::collections::HashMap;
use std::io::prelude::*;
use std::{fmt::Debug, io::Cursor};
use std::fs::File;
use regex::Regex;
use lazy_static::lazy_static;

use crate::texture::{TextureResource, EmbeddedTexture};

/** 
*  information for data.win file format taken from:
*  https://pcy.ulyssis.be/undertale/unpacking-corrected
*  
*  Datatype code mostly translated from:
*  https://github.com/BenjaminUrquhart/GameMaker-Parser
**/

pub struct DataFile {
	container: DataContainer,
	tpag: Vec<TextureResource>,
	txtr: Vec<EmbeddedTexture>
}

impl DataFile {
	pub fn new(mut file: File) -> Result<Self, String> {
		let mut buf = Vec::new();
		file.read_to_end(&mut buf);
		let container = DataContainer::new(buf)?;

		let txtr_chunk = container.chunk_map.get("txtr").unwrap();
		

		Ok(DataFile {
			container,
			txtr: Vec::new(),
			tpag: Vec::new()
		})
	}
}

#[derive(Clone, Debug)]
pub struct DataContainer {
	pub chunk_map: HashMap<String, DataChunk>,
	chunks: Vec<DataChunk>
}

impl DataContainer {
	pub fn new(contents: Vec<u8>) -> Result<Self, String> {
		let file_size = contents.len();
		let mut reader = Cursor::new(contents.as_slice());
		let mut total_read = 0;

		let mut file = DataContainer {
			chunk_map: HashMap::new(),
			chunks: Vec::new()
		};

		if file_size < 8 {
			return Err("Data file is too small to be valid".to_owned());
		}

		while total_read < file_size as u64{
			let offset = total_read;
			let mut buf = [0; 4];

			match reader.read_exact(&mut buf) {
				Ok(_) => {},
				Err(e) => return Err(format!("Error reading from file: {}", e))
			}

			total_read += 4;
			
			let chunk_id = match String::from_utf8(buf.to_vec()) {
				Ok(val) => val,
				Err(e) => return Err(format!("Chunk id is invalid UTF-8: {}", e))
			};

			lazy_static!{
				static ref TEST: Regex = Regex::new("\\w{4}").unwrap();
			}

			if chunk_id == "RASP" {
					break;
			}

			if !TEST.is_match(&chunk_id) {
				return Err("Chunk ID is invalid".to_owned())
			}

			buf = [0; 4];
			match reader.read_exact(&mut buf) {
				Ok(_) => {},
				Err(e) => return Err(format!("Error reading from file: {}", e))
			}
			total_read += 4;

			println!("{:?}", chunk_id);

			if chunk_id.matches("\\w{4}").count() > 0 {
				return Err(format!("I think this is an invalid chunk id: {}", chunk_id))
			};

			let mut chunk_size: i32 = ((buf[3] as i32 & 0xff)<<24)|((buf[2] as i32 & 0xff)<<16)|((buf[1] as i32 & 0xff)<<8)|(buf[0] as i32 & 0xff);
			if chunk_size.is_negative() {
				chunk_size = chunk_size.reverse_bits()
			}

			if chunk_size.is_negative() {
				return Err(format!("Chunk size for chunk {} is negative regardless of endieness", chunk_id));
			}

			println!("size: {:010X}", chunk_size);

			let mut buf = vec![0; chunk_size as usize];
			match reader.read_exact(&mut buf) {
				Ok(_) => {},
				Err(e) => return Err(format!("Error reading chunk from file: {}", e))
			}
			total_read += chunk_size as u64;

			println!("{}", buf.len());

			let chunk = DataChunk::new(chunk_id.clone(), buf.to_vec(), offset);
			file.chunk_map.insert(chunk_id, chunk.clone());
			file.chunks.push(chunk);
		}

		Ok(file)
	}
}

#[derive(Clone)]
pub struct DataChunk {
	type_id: String,
	contents: Vec<u8>,
	offset: u64,

	sub_chunks: Option<DataContainer>,
	parent: Option<Box<DataChunk>>,
}

impl DataChunk {
	pub fn new(id: String, contents: Vec<u8>, offset: u64) -> Self {
		let mut chunk = DataChunk {
			type_id: id,
			contents: contents.clone(),
			offset: offset,
			sub_chunks: None,
			parent: None
		};

		match DataContainer::new(contents) {
			Ok(file) => {
				chunk.sub_chunks = Some(file)
			},
			Err(e) => println!("Error reading sub chunks: {}", e)
		};

		chunk
	}
}

impl Debug for DataChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{type_id: {}, offset: {}, sub_chunks: {:?}, parent: {:?}}}", self.type_id, self.offset, self.sub_chunks, self.parent))
    }
}