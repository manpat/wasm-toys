use common::*;

#[derive(Debug)]
pub struct ToyFile {
	pub scenes: Vec<SceneData>,
	pub entities: Vec<EntityData>,
	pub meshes: Vec<MeshData>,
}

#[derive(Debug)]
pub struct SceneData {
	pub name: String,
	pub entities: Vec<u16>
}

#[derive(Debug)]
pub struct EntityData {
	pub name: String,
	pub mesh_id: u16,

	pub position: Vec3,
	pub rotation: Quat,
	pub scale: Vec3,
}

#[derive(Debug)]
pub struct MeshData {
	pub positions: Vec<Vec3>,
	pub indices: MeshIndices,
	pub color_data: Vec<MeshColorData>,
}

#[derive(Debug)]
pub enum MeshIndices {
	U8(Vec<u8>),
	U16(Vec<u16>),
}

#[derive(Debug)]
pub struct MeshColorData {
	pub name: String,
	pub data: Vec<Vec4>,
}

// TODO: entity queries
// TODO: mesh building