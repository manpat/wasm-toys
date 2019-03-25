use crate::imports::gl;
use common::math::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextureID(usize);

pub const NULL_TEXTURE: TextureID = TextureID(0);

pub struct TextureInfo {
	// name?
	pub gl_id: gl::TextureID,
	pub size: Vec2i,
}

pub struct TextureRegistry {
	textures: Vec<TextureInfo>,
}

impl TextureRegistry {
	pub fn new() -> Self {
		TextureRegistry {
			textures: Vec::new(),
		}
	}

	pub fn register_texture(&mut self, gl_id: gl::TextureID, size: Vec2i) -> TextureID {
		let item = self.textures.iter()
			.position(|x| x.gl_id == gl_id);

		if let Some(position) = item {
			assert!(self.textures[position].size == size);
			return TextureID(position);
		}

		let new_tex = TextureInfo { gl_id, size };

		console_log!("Texture registered {} {:?}", gl_id.0, size);

		let id = self.textures.len();
		self.textures.push(new_tex);
		TextureID(id)
	}

	pub fn get_texture_info(&self, id: TextureID) -> Option<&TextureInfo> {
		if id.0 == 0 { return None }
		self.textures.get(id.0 - 1)
	}
}