pub mod vertex;
pub mod mesh;
pub mod camera;
pub mod texture;
pub mod shader;
pub mod framebuffer;

pub use self::shader::Shader;
pub use self::texture::{Texture, TextureBuilder};
pub use self::mesh::{BasicDynamicMesh, DynamicMesh, MeshBuilding};
pub use self::camera::Camera;
pub use self::framebuffer::Framebuffer;

pub use crate::imports::gl;

pub fn create_shader(vsrc: &str, fsrc: &str, attribs: &[&str]) -> gl::ProgramID {
	unsafe {
		let program = gl::create_shader_program();

		let vsh = gl::create_shader(gl::ShaderType::Vertex, vsrc.into());
		let fsh = gl::create_shader(gl::ShaderType::Fragment, fsrc.into());

		for (i, &a) in attribs.iter().enumerate() {
			gl::bind_attrib_location(program, a.into(), i as u32);
		}

		gl::link_program(program, vsh, fsh);

		program
	}	
}

pub fn create_shader_combined(src: &str, attribs: &[&str]) -> gl::ProgramID {
	unsafe {
		let mut src = src.split("/* @@@ */");
		let (vsrc, fsrc) = (src.next().unwrap(), src.next().unwrap());

		let program = gl::create_shader_program();

		let vsh = gl::create_shader(gl::ShaderType::Vertex, vsrc.into());
		let fsh = gl::create_shader(gl::ShaderType::Fragment, fsrc.into());

		for (i, &a) in attribs.iter().enumerate() {
			gl::bind_attrib_location(program, a.into(), i as u32);
		}

		gl::link_program(program, vsh, fsh);

		program
	}	
}


pub struct StencilParams {
	pub condition: gl::StencilCondition,
	pub reference: u8,

	pub stencil_fail: gl::StencilOp,
	pub depth_fail: gl::StencilOp,
	pub pass: gl::StencilOp,
}

impl StencilParams {
	pub fn new(reference: u8) -> Self {
		StencilParams {
			reference,
			condition: gl::StencilCondition::Never,

			stencil_fail: gl::StencilOp::Keep,
			depth_fail: gl::StencilOp::Keep,
			pass: gl::StencilOp::Keep,
		}
	}


	pub fn pass_if(self, condition: gl::StencilCondition) -> Self {
		StencilParams { condition, ..self }
	}

	pub fn always(self) -> Self { self.pass_if(gl::StencilCondition::Always) }
	pub fn never(self) -> Self { self.pass_if(gl::StencilCondition::Never) }
	pub fn equal(self) -> Self { self.pass_if(gl::StencilCondition::Equal) }
	pub fn less_than_stencil(self) -> Self { self.pass_if(gl::StencilCondition::Less) }
	pub fn greater_than_stencil(self) -> Self { self.pass_if(gl::StencilCondition::Greater) }

	pub fn replace(self) -> Self {
		Self { pass: gl::StencilOp::Replace, ..self }
	}

	pub fn increment(self) -> Self {
		Self { pass: gl::StencilOp::Incr, ..self }
	}

	pub fn decrement(self) -> Self {
		Self { pass: gl::StencilOp::Decr, ..self }
	}

	pub fn invert(self) -> Self {
		Self { pass: gl::StencilOp::Invert, ..self }
	}
}

pub fn set_stencil(params: crate::graphics::StencilParams) {
	unsafe {
		gl::stencil_func(params.condition, params.reference, 0xff);
		gl::stencil_op(params.stencil_fail, params.depth_fail, params.pass);
	}
}

pub fn set_color_write(enabled: bool) {
	unsafe {
		gl::color_mask(enabled, enabled, enabled, enabled);
	}
}

pub fn set_depth_write(enabled: bool) {
	unsafe {
		gl::depth_mask(enabled);
	}
}

pub fn set_stencil_write(enabled: bool) {
	unsafe {
		gl::stencil_mask(if enabled { 0xFF } else { 0 });
	}
}