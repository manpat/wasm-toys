use crate::imports::gl;
use common::*;

pub struct Shader {
	gl_handle: gl::ProgramID,
	num_attribs: u32
}

impl Shader {
	pub fn from_combined(src: &str, attribs: &[&str]) -> Shader {
		unsafe {
			let mut src = src.split("/* @@@ */");
			let (vsrc, fsrc) = (src.next().unwrap(), src.next().unwrap());

			let gl_handle = gl::create_shader_program();

			let vsh = gl::create_shader(gl::ShaderType::Vertex, vsrc.into());
			let fsh = gl::create_shader(gl::ShaderType::Fragment, fsrc.into());

			for (i, &a) in attribs.iter().enumerate() {
				gl::bind_attrib_location(gl_handle, a.into(), i as u32);
			}

			gl::link_program(gl_handle, vsh, fsh);

			Shader {
				gl_handle,
				num_attribs: attribs.len() as u32,
			}
		}	
	}

	pub fn bind(&self) {
		unsafe {
			for i in 0..self.num_attribs {
				gl::enable_attribute(i);
			}

			for i in self.num_attribs..8 {
				gl::disable_attribute(i);
			}

			gl::use_program(self.gl_handle);
		}
	}

	pub fn set_uniform<U: UniformType>(&self, name: &str, data: U) {
		data.apply(self.gl_handle, name);
	}
}


pub trait UniformType {
	fn apply(&self, gl_handle: gl::ProgramID, name: &str);
}

impl UniformType for u32 {
	fn apply(&self, gl_handle: gl::ProgramID, name: &str) {
		unsafe {
			gl::set_uniform_int_raw(gl_handle, name.into(), *self);
		}
	}
}

impl UniformType for f32 {
	fn apply(&self, gl_handle: gl::ProgramID, name: &str) {
		unsafe {
			gl::set_uniform_f32_raw(gl_handle, name.into(), *self);
		}
	}
}

impl UniformType for Vec4 {
	fn apply(&self, gl_handle: gl::ProgramID, name: &str) {
		unsafe {
			gl::set_uniform_vec4_raw(gl_handle, name.into(), self.x, self.y, self.z, self.w);
		}
	}
}

impl UniformType for Color {
	fn apply(&self, gl_handle: gl::ProgramID, name: &str) {
		self.to_vec4().apply(gl_handle, name);
	}
}

impl UniformType for Mat4 {
	fn apply(&self, gl_handle: gl::ProgramID, name: &str) {
		unsafe {
			gl::set_uniform_mat4_raw(gl_handle, name.into(), &self.transpose() as *const Mat4);
		}
	}
}