#[cfg(target_os="macos")]
pub mod mac_lib {
	pub use install_file;
	pub use gen_install_data;
	pub use is_installed;
}