use crate::imports::gl;
use crate::graphics::vertex::{Vertex, Descriptor};

pub struct DynamicMesh<T: Vertex> {
	vertices: Vec<T>,
	indices: Vec<u16>,
	descriptor: Descriptor,

	vbo: gl::BufferID,
	ebo: gl::BufferID,
}


impl<T: Vertex> DynamicMesh<T> {
	pub fn new() -> Self {
		unsafe {
			DynamicMesh {
				vertices: Vec::new(),
				indices: Vec::new(),
				descriptor: T::descriptor(),

				vbo: gl::create_buffer(),
				ebo: gl::create_buffer(),
			}
		}
	}

	pub fn draw(&self, dm: gl::DrawMode) {
		use std::mem::size_of;

		unsafe {
			if self.vertices.len() > 0 && self.indices.len() > 0 {
				gl::bind_buffer(gl::BufferTarget::ArrayBuffer, self.vbo);
				gl::upload_buffer_data(gl::BufferTarget::ArrayBuffer,
					self.vertices.as_ptr() as *const u8,
					self.vertices.len() * size_of::<T>());

				self.descriptor.bind();

				gl::bind_buffer(gl::BufferTarget::ElementArrayBuffer, self.ebo);
				gl::upload_buffer_data(gl::BufferTarget::ElementArrayBuffer,
					self.indices.as_ptr() as *const u8,
					self.indices.len() * size_of::<u16>());

				gl::draw_elements(dm, self.indices.len(), gl::Type::UnsignedShort, 0);
			}
		}
	}

	pub fn clear(&mut self) {
		self.vertices.clear();
		self.indices.clear();
	}

	pub fn add_geometry(&mut self, verts: &[T], indices: &[u16]) {
		let start = if let Some(s) = self.vert_start() { s } else { return };

		self.vertices.extend_from_slice(verts);
		self.indices.extend(indices.iter().map(|i| i + start));
	}

	pub fn add_quad(&mut self, verts: &[T]) {
		let start = if let Some(s) = self.vert_start() { s } else { return };
		
		let es = [
			start + 0, start + 1, start + 2,
			start + 0, start + 2, start + 3
		];

		self.vertices.extend_from_slice(verts);
		self.indices.extend_from_slice(&es);
	}

	fn vert_start(&self) -> Option<u16> {
		let start = self.vertices.len();
		if start > 0xffff {
			console_warn!("Too many verts!");
			return None;
		}

		Some(start as u16)
	}
}