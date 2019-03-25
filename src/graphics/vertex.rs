use common::math::*;


pub trait Vertex : Copy {
	fn descriptor() -> Descriptor;
} 


struct AttributeBinding {
	position: u32,
	components: u32, // assume floats
}

pub struct Descriptor {
	bindings: Vec<AttributeBinding>,
	stride: usize,
}

impl Descriptor {
	pub fn new() -> Self {
		Descriptor{ bindings: Vec::new(), stride: 0 }
	}

	pub fn from(comps: &[u32]) -> Self {
		let mut bindings = Vec::with_capacity(comps.len());

		for (i, &cs) in comps.iter().enumerate() {
			bindings.push(AttributeBinding{
				position: i as u32,
				components: cs
			});
		}

		let stride = comps.iter().sum::<u32>() as usize * 4;

		Descriptor{ bindings, stride }
	}

	pub fn add(mut self, position: u32, components: u32) -> Self {
		self.bindings.push(AttributeBinding{ position, components });
		self.stride += components as usize * 4;
		self
	}

	pub fn bind(&self) {
		use crate::imports::gl;

		let mut offset = 0;

		for binding in self.bindings.iter() {
			unsafe {
				gl::vertex_attrib_pointer(
					binding.position, binding.components,
					gl::Type::Float, false,
					self.stride, offset);			
			}
			
			offset += binding.components as usize * 4;
		}
	}
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ColorVertex {
	pos: Vec3,
	color: Vec3,
}

impl ColorVertex {
	pub fn new(pos: Vec3, color: Vec3) -> Self {
		ColorVertex{pos, color}
	}
}

impl Vertex for ColorVertex {
	fn descriptor() -> Descriptor {
		Descriptor::from(&[3, 3])
	}
}



#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TexturedVertex {
	pos: Vec3,
	uv: Vec2,
}

impl TexturedVertex {
	pub fn new(pos: Vec3, uv: Vec2) -> Self {
		TexturedVertex{pos, uv}
	}
}

impl Vertex for TexturedVertex {
	fn descriptor() -> Descriptor {
		Descriptor::from(&[3, 2])
	}
}
