use std::env;
use std::path::{Path, PathBuf};
use std::fs::{read, create_dir_all, File};
use std::io::{Cursor, copy};

use configparser::ini::Ini;

use zip::ZipArchive;
use zip::read::ZipFile;

use serde::Deserialize;

#[derive(Deserialize)]
struct ModMeta {
	name: String,
	author: String,
	version: u8,
	palette: bool,
	music: bool
}

#[derive(Deserialize)]
struct Palette {
	author: String,
	name: String,
	colors: Vec<String>
}

const PROGRAM_VERSION: &str = "v0.1"; 
const MXMOD_FORMAT_VERSION: u8 = 1;

fn main() {
	println!("Mixolumia mod installer {}", PROGRAM_VERSION);

	let mut args: Vec<String> = env::args().collect();
	args.remove(0);

	let mxmod_file = match args.get(0) {
		Some(val) => val,
		None => {
			println!("You have to run with either a .mxmod, .mxpalette, or a .mxmusic file as a parameter");
			return;
		}
	};

	let file_contents = match read(mxmod_file) {
		Ok(val) => val,
		Err(e) => {
			println!("Error reading file {}, {}", mxmod_file, e);
			return;
		}
	};
	
	match mxmod_file.split(".").last() {
		Some("mxmod") => install_mxmod_file(file_contents, mxmod_file.to_owned()),
		Some("mxpalette") => {
			let json = String::from_utf8_lossy(&file_contents).into_owned();
			match serde_json::from_str::<Palette>(&json) {
				Ok(val) => install_mxpalette_file(val),
				Err(e) => println!("Error parsing .mxpalette json: {}", e)
			}
		},
		Some("mxmusic") => install_mxmusic_file(file_contents, mxmod_file.to_owned()),
		Some(_) => {},
		None => {}
	}	
}

fn install_mxmod_file(mod_file_contents: Vec<u8>, file_name: String) {
	println!("Attempting to install mod {}", file_name);

	// Load in zip archive
	let mut mod_zip = ZipArchive::new(Cursor::new(mod_file_contents)).expect("Error opening mxmod file");

	// Load in mod_meta.json
	let mod_meta = match mod_zip.clone().by_name("mod_meta.json") {
		Ok(mut mod_meta_file) => {
			let mut file_contents = Vec::new();
			copy(&mut mod_meta_file, &mut file_contents).expect("Error extracting contents of mod_meta.json");

			serde_json::from_slice::<ModMeta>(&file_contents).expect("mod_meta.json is invalid")
		}
		Err(_) => {
			println!("Error: No mod_meta.json found in mxmod");
			return;
		}
	};

	// Version check
	if mod_meta.version > MXMOD_FORMAT_VERSION {
		println!("Error: mxmod version is newer than what this installer supports (1\nexiting...");
		return;
	}

	println!("Installing mod '{} - {}'", mod_meta.name, mod_meta.author);
	
	// Install music packs
	if mod_meta.music {
		let music_files =  mod_zip.file_names().filter(|name| name.starts_with("music/") && name.split(".").last().unwrap() == "mxmusic").map(|f| f.to_owned()).collect::<Vec<String>>();
		let music_folder = mod_zip.by_name("music/");

		// Make sure music dir exists and is a dir
		if music_folder.is_ok() {
			if music_folder.unwrap().is_dir() {

				println!("Attempting to install music packs");
				
				// Loop over mxmusic files and install them all
				for file in music_files {
					match mod_zip.by_name(&file) {
						Ok(mut music_file) => {
							let mut music_file_data = Vec::new();
							copy(&mut music_file, &mut music_file_data).expect("Error reading data of mxmusic file");
							install_mxmusic_file(music_file_data, file);
						},
						Err(_) => {}
					}
				}

				println!("Installed music packs");
			}
		} else {
			println!("Mod says that music is included but `music` folder is not present\nSkipping installing music packs...")
		}
	}

	// Install palettes
	if mod_meta.palette {
		// Grab list of all mxpalette files
		let palette_files =  mod_zip.file_names().filter(|name| name.starts_with("palettes/") && name.split(".").last().unwrap() == "mxpalette").map(|f| f.to_owned()).collect::<Vec<String>>();
		let palettes_folder = mod_zip.by_name("palettes/");

		// Make sure palettes dir exists and is a dir
		if palettes_folder.is_ok() {
			if palettes_folder.unwrap().is_dir() {

				// Loop over mxpalette files and install them all
				for file in palette_files {
					match mod_zip.by_name(&file) {
						Ok(mut palette_file) => {
							let mut palette_file_data = Vec::new();
							copy(&mut palette_file, &mut palette_file_data).expect("Error reading palette file");

							match serde_json::from_slice::<Palette>(&palette_file_data) {
								Ok(pal) => install_mxpalette_file(pal),
								Err(e) => println!("Error reading mxpalette file: {}", e)
							}
						},
						Err(_) => println!("File doesn't exist?")
					}
				}
			}
		} else {
			println!("Mod says that palettes are included but `palettes` folder is not present\nSkipping installing palettes...")
		}
	}

	println!("Finished installing '{} - {}'", mod_meta.name, mod_meta.author);
}

fn install_mxpalette_file(palette: Palette) {
	println!("Attempting to install palette '{} - {}'", palette.name, palette.author);
	// Get the path to user_palettes.ini
	let mixolumia_path = get_mixolumia_dir();
	let palette_file = mixolumia_path.join("user_palettes.ini");

	// load in the ini
	let mut palette_ini = Ini::new();
	palette_ini.load(palette_file.to_str().unwrap()).expect("Error reading user_palettes.ini");

	// Calculate the next palette key
	let mut keys =  palette_ini.get_map().unwrap().keys().map(|k| k.parse::<u16>().unwrap()).collect::<Vec<u16>>();
	keys.sort();
	let new_palette_key = keys.last().unwrap() + 1_u16;
	

	// Set the new palette's colors
	for i in  0..palette.colors.len() {
		palette_ini.set(&new_palette_key.to_string(), &i.to_string(), Some(palette.colors.get(i).unwrap().to_owned()));
	}

	// Set the author and name
	palette_ini.set(&new_palette_key.to_string(), "author", Some(palette.author.clone()));
	palette_ini.set(&new_palette_key.to_string(), "name", Some(palette.name.clone()));

	// Write the modified ini to disk
	palette_ini.write(palette_file.to_str().unwrap()).expect("Error writing user_palettes.ini");
	println!("Installed palette '{} - {}' successfully", &palette.name, &palette.author);
}

fn install_mxmusic_file(music_file_contents: Vec<u8>, file_name: String) {
	// ensure_data_dir_exists();

	// Load zip archive
	let mut music_zip = match ZipArchive::new(Cursor::new(music_file_contents)) {
		Ok(val) => val,
		Err(e) => {
			println!("Error reading .mxmusic contents: '{}'", e);
			return;
		}
	};

	// Get where to install to
	let pack_name = Path::new(&file_name).file_stem().unwrap().to_str().unwrap().to_owned();
	let install_dir = get_mixolumia_dir().join(format!("data/music/{}/", pack_name));
	println!("Attempting to install music pack '{}'", pack_name);

	// Extract all files to install dir
	for i in 0..music_zip.len() {
		let mut file = music_zip.by_index(i).expect("Error reading .mxmusic file contents");
		extract_file(install_dir.clone(), &mut file);
	}

	println!("Successfully installed music pack '{}'", pack_name);
}

/// Gets the directory to install mixolumia files to
fn get_mixolumia_dir() -> PathBuf {
	#[cfg(windows)]
	let mixolumia_path = format!("{}\\Mixolumia", env::var("LOCALAPPDATA").expect("Error reading LOCALAPPDATA"));
	#[cfg(unix)]
	let mixolumia_path = format!("{}/Library/Application Support/com.davemakes.mixolumia/", env::var("HOME").expect("Error reading HOME"));
	
	Path::new(&mixolumia_path).to_owned()
}


// /// Ensures the data directory exists so we don't accidentally
// fn ensure_data_dir_exists() {
// 	let mixolumia_dir = get_mixolumia_dir();
// 	let data_dir = mixolumia_dir.join("data");
// 	if !data_dir.exists() {
// 		create_dir_all(data_dir).expect("Error creating mixolumia data directory");
// 	}
// }

/// Extracts a file from a zip to a location
fn extract_file(path: PathBuf, file: &mut ZipFile) {
	if file.is_dir() {

	} else {
		let p = path.join(file.sanitized_name());
		
		let parent = p.parent().expect("Error getting parent dir");

		if !parent.exists() {
			create_dir_all(path.join(parent)).expect("Error creating parent dir")
		}

		let mut outfile = File::create(path.join(p)).expect("Error creating output");
		copy(file, &mut outfile).expect("Error writing file contents to outfile");
	}
}