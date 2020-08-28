use std::io::Error;
use std::ffi::OsStr;
use std::ffi::CString;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::null_mut;
use winapi::um::winuser::{MB_YESNO, MessageBoxW};
use winreg::RegKey;
use winreg::enums::*;

pub fn yes_no_box(msg: &str) -> Result<bool, Error> {
	let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
	let ret = unsafe {
		MessageBoxW(null_mut(), wide.as_ptr(), wide.as_ptr(), MB_YESNO)
	};
	if ret == 0 { Err(Error::last_os_error()) }
	else { Ok(ret == 6 ) }
}

pub fn register_file_types() -> std::io::Result<()> {
	register_file_type(".mxmod")?;
	register_file_type(".mxmusic")?;
	register_file_type(".mxpalette")?;

	let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
	let path = Path::new("MixolumaModInstaller.install");
	let (key, _) = hkcr.create_subkey(&path)?;

	key.set_value("AppUserModelID", &"Maddymakesgames.MixolumiaModInstaller")?;
	key.set_value("", &"Mixolumia mod file")?;

	let (key, _) = hkcr.create_subkey(&path.join("DefaultIcon"))?;
	key.set_value("", &Path::new(&std::env::var("LOCALAPPDATA").unwrap()).join("MixolumiaModInstaller").join("icon.ico").to_str().unwrap())?;

	let (key, _) = hkcr.create_subkey(&path.join("shell").join("open").join("command"))?;

	key.set_value("", &format!(r#""{}" %1"#, &std::env::current_exe().unwrap().to_str().unwrap()))?;
	

	Ok(())
}

fn register_file_type(file_type: &str) -> std::io::Result<()> {
	let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
	let path = Path::new(file_type);
	let (key, _) = hkcr.create_subkey(&path)?;

	key.set_value("", &"MixolumaModInstaller.install")?;

	Ok(())
}

pub fn elevate() {
	unsafe {
		winapi::um::shellapi::ShellExecuteA(
			core::ptr::null_mut(), 
			CString::new("runas").unwrap().as_ptr(),
			CString::new(std::env::current_exe().unwrap().to_str().unwrap()).unwrap().as_ptr(),
			CString::new("--register_extensions").unwrap().as_ptr(),
			core::ptr::null_mut(),
			winapi::um::winuser::SW_SHOWNORMAL
		);
	}
}

pub fn is_elevated() -> bool {
	let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
	let path = Path::new("MixolumaModInstaller.install");
	match hkcr.create_subkey(&path.join("perms_check")) {
		Ok(_) => {
			hkcr.delete_subkey(&path.join("perms_check")).unwrap();
			true
		},
		Err(_) => false
	}
}

pub fn test_registry() -> bool {

	match test_key(".mxmod") {
		Ok(true) => {},
		Err(_) |
		Ok(false) => return false
	}

	match test_key(".mxmusic") {
		Ok(true) => {},
		Err(_) |
		Ok(false) => return false
	}

	match test_key(".mxpalette") {
		Ok(true) => {},
		Err(_) |
		Ok(false) => return false
	}

	let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

	match hkcr.open_subkey(&Path::new("MixolumaModInstaller.install").join("shell").join("open").join("command")) {
		Ok(key) => {
			match key.get_value::<String, &str>("") {
				Ok(val) => {
					val == format!(r#""{}" %1"#, &std::env::current_exe().unwrap().to_str().unwrap())
				},
				Err(_) => false
			}
		},
		Err(_) => false
	};

	false
}

fn test_key(file_type: &str) -> std::io::Result<bool> {
	let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
	let path = Path::new(file_type);
	let key= hkcr.open_subkey(&path)?;

	let value: String = key.get_value("")?;
	Ok(value == "MixolumaModInstaller.install")
}