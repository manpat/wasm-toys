use crate::imports::gl;
use common::math::*;

pub struct Texture {
	pub(crate) gl_handle: gl::TextureID,
	format: gl::Format,
	component_type: gl::Type,
	size: Vec2i,
}

impl Texture {
	pub fn new(format: gl::Format, component_type: gl::Type) -> Texture {
		unsafe {
			Texture {
				gl_handle: gl::create_texture(),
				format, component_type,
				size: Vec2i::zero()
			}
		}
	}

	pub fn bind(&self, slot: i32) {
		unsafe {
			gl::active_texture(slot);
			gl::bind_texture(self.gl_handle);
		}
	}

	pub fn reserve(&mut self, size: Vec2i) {
		self.bind(0);
		self.size = size;

		unsafe {
			gl::upload_image_data(
				size.x as _, size.y as _,
				self.format, self.component_type,
				std::ptr::null(), 0
			);
		}
	}

	pub fn upload<T: Copy>(&mut self, size: Vec2i, data: &[T]) {
		self.bind(0);
		self.size = size;

		unsafe {
			gl::upload_image_data(size.x as _, size.y as _,
				self.format, self.component_type,
				data.as_ptr() as _, std::mem::size_of::<T>() * data.len());
		}
	}

	pub fn format(&self) -> gl::Format { self.format }
	pub fn component_type(&self) -> gl::Type { self.component_type }
	pub fn size(&self) -> Vec2i { self.size }
}


pub struct TextureBuilder {
	filter: gl::TextureParamValue,
	wrap: gl::TextureParamValue,
	format: gl::Format,
	comp_type: gl::Type,
}

impl TextureBuilder {
	pub fn new() -> TextureBuilder {
		TextureBuilder {
			filter: gl::TextureParamValue::Nearest,
			wrap: gl::TextureParamValue::ClampToEdge,
			format: gl::Format::RGBA,
			comp_type: gl::Type::UnsignedByte,
		}
	}

	pub fn r8(self) -> TextureBuilder {
		TextureBuilder { format: gl::Format::Luminance, comp_type: gl::Type::UnsignedByte, ..self }
	}

	pub fn rgb8(self) -> TextureBuilder {
		TextureBuilder { format: gl::Format::RGB, comp_type: gl::Type::UnsignedByte, ..self }
	}

	pub fn rgba8(self) -> TextureBuilder {
		TextureBuilder { format: gl::Format::RGBA, comp_type: gl::Type::UnsignedByte, ..self }
	}

	pub fn nearest(self) -> TextureBuilder {
		TextureBuilder { filter: gl::TextureParamValue::Nearest, ..self }
	}
	pub fn linear(self) -> TextureBuilder {
		TextureBuilder { filter: gl::TextureParamValue::Linear, ..self }
	}

	pub fn clamp(self) -> TextureBuilder {
		TextureBuilder { wrap: gl::TextureParamValue::ClampToEdge, ..self }
	}
	pub fn repeat(self) -> TextureBuilder {
		TextureBuilder { wrap: gl::TextureParamValue::Repeat, ..self }
	}

	pub fn build(self) -> Texture {
		use gl::TextureParam;

		let texture = Texture::new(self.format, self.comp_type);
		texture.bind(0);

		unsafe {
			gl::tex_parameter(TextureParam::MinFilter, self.filter);
			gl::tex_parameter(TextureParam::MagFilter, self.filter);
			gl::tex_parameter(TextureParam::WrapS, self.wrap);
			gl::tex_parameter(TextureParam::WrapT, self.wrap);
		}

		texture
	}
}