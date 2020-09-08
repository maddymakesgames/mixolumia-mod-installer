

/// Represents a TPAG resource
pub struct TextureResource {
	x: u16,
	y: u16,
	width: u16,
	height: u16,
	render_x: u16,
	render_y: u16,
	bounding_x: u16,
	bounding_y: u16,
	bounding_width: u16,
	bounding_height: u16,
	spritesheet_id: u16
}

/// Represents a TXTR resource
pub struct EmbeddedTexture {
	/// Pointer to ImageData
	pub image_contents: u32,
	scaled: u32,
	generated_mips: u32
}

impl EmbeddedTexture {
	pub fn from_buf(buf: &mut Vec<u8>) -> Self {
		let scaled = ((buf.remove(0) as u32 & 0xff)<<24)|((buf.remove(0) as u32 & 0xff)<<16)|((buf.remove(0) as u32 & 0xff)<<8)|(buf.remove(0) as u32 & 0xff);
		let generated_mips = ((buf.remove(0) as u32 & 0xff)<<24)|((buf.remove(0) as u32 & 0xff)<<16)|((buf.remove(0) as u32 & 0xff)<<8)|(buf.remove(0) as u32 & 0xff);
		let image_contents = ((buf.remove(0) as u32 & 0xff)<<24)|((buf.remove(0) as u32 & 0xff)<<16)|((buf.remove(0) as u32 & 0xff)<<8)|(buf.remove(0) as u32 & 0xff);
		EmbeddedTexture {
			image_contents,
			scaled,
			generated_mips
		}
	}
}

pub type ImageData = Vec<u8>;