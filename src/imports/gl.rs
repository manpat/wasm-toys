use crate::imports::RawStr;
use common::math::{Mat4, Vec4};

#[repr(u32)]
pub enum ShaderType {
	Vertex,
	Fragment
}

#[repr(u32)]
pub enum BufferTarget {
	ArrayBuffer = 34962,
	ElementArrayBuffer = 34963,
}

#[repr(u32)]
pub enum Type {
	Byte = 5120,
	Short = 5122,
	UnsignedByte = 5121,
	UnsignedShort = 5123,
	Float = 5126,
}

#[repr(u32)]
pub enum Format {
	RGB = 6407,
	RGBA = 6408,
}

#[repr(u32)]
pub enum TextureParam {
	MagFilter = 10240,
	MinFilter = 10241,
	WrapS = 10242,
	WrapT = 10243,
}

#[repr(u32)]
pub enum TextureParamValue {
	Nearest = 9728,
	Linear = 9729,
	NearestMipmapNearest = 9984,
	LinearMipmapNearest = 9985,
	NearestMipmapLinear = 9986,
	LinearMipmapLinear = 9987,

	Repeat = 10497,
	ClampToEdge = 33071,
	MirroredRepeat = 33648,
}

#[repr(u32)]
pub enum DrawMode {
	Points = 0,
	Lines = 1,
	LineLoop = 2,
	LineStrip = 3,
	Triangles = 4,
	TriangleStrip = 5,
	TriangleFan = 6,
}

#[repr(u32)]
pub enum Capability {
	Blend = 3042,
	CullFace = 2884,
	DepthTest = 2929,
	Dither = 3024,
	PolygonOffsetFill = 32823,
	SampleAlphaToCoverage = 32926,
	SampleCoverage = 32928,
	ScissorTest = 3089,
	StencilTest = 2960
}

#[repr(u32)]
pub enum StencilCondition {
	Never = 512,
	Always = 519,
	Equal = 514,
	Less = 513,
	Greater = 516,
}

#[repr(u32)]
pub enum StencilOp {
	Keep = 7680,
	Replace = 7681,
	Incr = 7682,
	Decr = 7683,
	Invert = 5386,
}

#[repr(u32)]
pub enum BlendFactor {
	Zero = 0,
	One = 1,

	SrcColor = 768,
	OneMinusSrcColor = 769,
	DstColor = 774,
	OneMinusDstColor = 775,

	SrcAlpha = 770,
	OneMinusSrcAlpha = 771,
	DstAlpha = 772,
	OneMinusDstAlpha = 773,

	ConstantColor = 32769,
	OneMinusConstantColor = 32770,
	ConstantAlpha = 32771,
	OneMinusConstantAlpha = 32772,
}

pub const COLOR_BUFFER_BIT: u32 = 1<<14;
pub const DEPTH_BUFFER_BIT: u32 = 1<<8;
pub const STENCIL_BUFFER_BIT: u32 = 1<<10;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ShaderID(pub u32);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ProgramID(pub u32);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BufferID(pub u32);

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextureID(pub u32);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FramebufferID(pub u32);

extern {
	pub fn viewport(x: i32, y: i32, w: i32, h: i32);
	pub fn scissor(x: i32, y: i32, w: i32, h: i32);
	pub fn clear_color(r: f32, g: f32, b: f32, a: f32);
	pub fn clear(_: u32);
	pub fn enable(_: Capability);
	pub fn disable(_: Capability);

	pub fn blend_func(src: BlendFactor, dst: BlendFactor);

	pub fn draw_arrays(_: DrawMode, start: usize, vert_count: usize);
	pub fn draw_elements(_: DrawMode, el_count: usize, el_type: Type, el_offset: usize /*bytes*/);

	pub fn create_buffer() -> BufferID;
	pub fn bind_buffer(_: BufferTarget, _: BufferID);
	pub fn upload_buffer_data(_: BufferTarget, _: *const u8, _: usize);
	pub fn vertex_attrib_pointer(attrib: u32, components: u32, _: Type, normalize: bool, stride: usize, offset: usize);
	pub fn enable_attribute(_: u32);
	pub fn disable_attribute(_: u32);

	pub fn create_texture() -> TextureID;
	pub fn bind_texture(_: TextureID);
	pub fn upload_image_data(w: u32, h: u32, _: Format, _: Type);
	pub fn tex_parameter(_: TextureParam, _: TextureParamValue);

	pub fn create_shader_program() -> ProgramID;
	pub fn create_shader(_: ShaderType, _: RawStr) -> ShaderID;
	pub fn bind_attrib_location(_: ProgramID, name: RawStr, idx: u32);
	pub fn link_program(_: ProgramID, vert: ShaderID, frag: ShaderID);
	pub fn use_program(_: ProgramID);

	pub fn stencil_func(_: StencilCondition, reference: u8, mask: u8);
	pub fn stencil_op(stencil_fail: StencilOp, depth_fail: StencilOp, pass: StencilOp);

	pub fn color_mask(r: bool, g: bool, b: bool, a: bool);
	pub fn depth_mask(enabled: bool);
	pub fn stencil_mask(bits: u8);

	fn set_uniform_int_raw(_: ProgramID, _: RawStr, _: u32);
	fn set_uniform_f32_raw(_: ProgramID, _: RawStr, _: f32);
	fn set_uniform_vec4_raw(_: ProgramID, _: RawStr, _: f32, _: f32, _: f32, _: f32);
	fn set_uniform_mat4_raw(_: ProgramID, _: RawStr, _: *const Mat4);
}

pub fn set_uniform_int(program: ProgramID, name: &str, i: u32) {
	unsafe {
		set_uniform_int_raw(program, name.into(), i);
	}
}

pub fn set_uniform_f32(program: ProgramID, name: &str, f: f32) {
	unsafe {
		set_uniform_f32_raw(program, name.into(), f);
	}
}

pub fn set_uniform_vec4(program: ProgramID, name: &str, v: Vec4) {
	unsafe {
		set_uniform_vec4_raw(program, name.into(), v.x, v.y, v.z, v.w);
	}
}

pub fn set_uniform_mat4(program: ProgramID, name: &str, v: &Mat4) {
	unsafe {
		set_uniform_mat4_raw(program, name.into(), &v.transpose() as *const Mat4);
	}
}

impl TextureID {
	pub fn is_valid(self) -> bool { self.0 != 0 }
}