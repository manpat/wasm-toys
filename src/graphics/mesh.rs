use crate::imports::gl;
use crate::graphics::vertex::{Vertex, Descriptor};

pub struct DynamicMesh<T: Vertex> {
	vertices: Vec<T>,
	indices: Vec<u16>,
	descriptor: Descriptor,

	vbo: gl::BufferID,
	ebo: gl::BufferID,
}

pub struct BasicDynamicMesh<T: Vertex> {
	vertices: Vec<T>,
	descriptor: Descriptor,
	vbo: gl::BufferID,
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

	pub fn apply<F>(&mut self, mut f: F) where F: FnMut(&mut T) {
		for v in self.vertices.iter_mut() {
			f(v);
		}
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


impl<T: Vertex + Copy> BasicDynamicMesh<T> {
	pub fn new() -> Self {
		unsafe {
			BasicDynamicMesh {
				vertices: Vec::new(),
				descriptor: T::descriptor(),
				vbo: gl::create_buffer(),
			}
		}
	}

	pub fn draw(&self, dm: gl::DrawMode) {
		use std::mem::size_of;

		unsafe {
			if self.vertices.len() > 0 {
				gl::bind_buffer(gl::BufferTarget::ArrayBuffer, self.vbo);
				gl::upload_buffer_data(gl::BufferTarget::ArrayBuffer,
					self.vertices.as_ptr() as *const u8,
					self.vertices.len() * size_of::<T>());

				self.descriptor.bind();

				gl::draw_arrays(dm, 0, self.vertices.len());
			}
		}
	}

	pub fn add_vertex(&mut self, vert: T) {
		self.vertices.push(vert);
	}

	pub fn add_vertices(&mut self, verts: &[T]) {
		self.vertices.extend_from_slice(verts);
	}
}




pub trait IntoIndex {
	fn into_index(self) -> u16;
}

impl IntoIndex for u16 {
	fn into_index(self) -> u16 { self }
}

impl<'a> IntoIndex for &'a u16 {
	fn into_index(self) -> u16 { *self }
}



pub trait MeshBuilding<T: Vertex> {
	fn add_geometry<I, Item>(&mut self, verts: &[T], indices: I) where I: IntoIterator<Item=Item>, Item: IntoIndex;
	fn clear(&mut self);

	fn add_quad(&mut self, verts: &[T]) {
		self.add_geometry(verts, &[0, 1, 2, 0, 2, 3]);
	}

	fn add_tri_fan(&mut self, vs: &[T]) {
		assert!(vs.len() >= 3);

		let indices = (1..vs.len()-1)
			.flat_map(|i| {
				let i = i as u16;
				let is = [0, i, i+1];
				(0..3).map(move |i| is[i])
			});

		self.add_geometry(vs, indices);
	}

	fn add_tri_strip(&mut self, vs: &[T]) {
		assert!(vs.len() >= 3);

		let indices = (0..vs.len()-2)
			.flat_map(|i| (0..3).map(move |offset| i as u16 + offset));

		self.add_geometry(vs, indices);
	}
}

impl<T: Vertex> MeshBuilding<T> for DynamicMesh<T> {
	fn add_geometry<I, Item>(&mut self, verts: &[T], indices: I) where I: IntoIterator<Item=Item>, Item: IntoIndex {
		let start = if let Some(s) = self.vert_start() { s } else { return };

		self.vertices.extend_from_slice(verts);
		self.indices.extend(indices.into_iter().map(|i| i.into_index() + start));
	}

	fn clear(&mut self) {
		self.vertices.clear();
		self.indices.clear();
	}
}

impl<T: Vertex> MeshBuilding<T> for BasicDynamicMesh<T> {
	fn add_geometry<I, Item>(&mut self, verts: &[T], indices: I) where I: IntoIterator<Item=Item>, Item: IntoIndex {
		self.vertices.extend(indices.into_iter().map(|i| verts[i.into_index() as usize]));
	}

	fn clear(&mut self) {
		self.vertices.clear();
	}
}