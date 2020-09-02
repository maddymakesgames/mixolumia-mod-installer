use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use std::env;

use configparser::ini::Ini;

mod main;

#[no_mangle]
pub extern "C" fn install_file(path_ptr: *const c_char) {
	let c_str = unsafe { CStr::from_ptr(path_ptr) };
	match c_str.to_str() {
		Err(_) => {},
		Ok(string) => crate::main::run(vec![string.to_owned()])
	};
}

#[no_mangle]
pub extern "C" fn gen_install_data() {
	crate::main::install();
}

#[no_mangle]
pub extern "C" fn is_installed() -> bool {
	let home_dir = env::var("HOME").expect("Error getting home directory");
	let install_dir = Path::new(&home_dir).join("MixolumiaModInstaller");
	
	if install_dir.exists() {
		let mut config_ini = Ini::new();
		match config_ini.load(install_dir.join("config.ini").to_str().unwrap()) {
			Ok(_) => match config_ini.get("metadata", "version") {
				Some(string) => string == crate::main::PROGRAM_VERSION,
				None => false
			},
			Err(_) => false
		}
	} else {
		false
	}
}