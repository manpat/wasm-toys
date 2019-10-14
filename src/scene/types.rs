use common::*;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct ToyFile {
	pub scenes: Vec<SceneData>,
	pub entities: Vec<EntityData>,
	pub meshes: Vec<MeshData>,
}

#[derive(Debug, Clone)]
pub struct SceneData {
	pub name: String,
	pub entities: Vec<u16>
}

#[derive(Debug, Clone)]
pub struct EntityData {
	pub name: String,
	pub mesh_id: u16,

	pub position: Vec3,
	pub rotation: Quat,
	pub scale: Vec3,
}

#[derive(Debug, Clone)]
pub struct MeshData {
	pub positions: Vec<Vec3>,
	pub indices: MeshIndices,
	pub color_data: Vec<MeshColorData>,
}

#[derive(Debug, Clone)]
pub enum MeshIndices {
	U8(Vec<u8>),
	U16(Vec<u16>),
}

#[derive(Debug, Clone)]
pub struct MeshColorData {
	pub name: String,
	pub data: Vec<Vec4>,
}

#[derive(Debug, Clone, Copy)]
pub struct SceneRef<'toy> {
	file: &'toy ToyFile,
	scene: &'toy SceneData,
}

#[derive(Debug, Clone, Copy)]
pub struct EntityRef<'toy> {
	file: &'toy ToyFile,
	entity: &'toy EntityData,
}

impl ToyFile {
	pub fn find_scene(&self, name: &str) -> Option<SceneRef> {
		self.scenes.iter()
			.find(|e| e.name == name)
			.map(|scene| SceneRef::from(self, scene))
	}

	pub fn find_entity(&self, name: &str) -> Option<EntityRef> {
		self.entities.iter()
			.find(|e| e.name == name)
			.map(|entity| EntityRef::from(self, entity))
	}
}

impl MeshData {
	pub fn color_data(&self, name: &str) -> Option<&MeshColorData> {
		self.color_data.iter()
			.find(|l| l.name == name)
	}
}

impl SceneRef<'_> {
	pub fn from<'t>(file: &'t ToyFile, scene: &'t SceneData) -> SceneRef<'t> {
		SceneRef { file, scene }
	}

	pub fn entities(&self) -> impl Iterator<Item=EntityRef> {
		let file = &self.file;

		self.scene.entities.iter()
			.map(move |&id| &file.entities[id as usize - 1])
			.map(move |entity| EntityRef::from(file, entity))
	}

	pub fn find_entity(&self, name: &str) -> Option<EntityRef> {
		self.entities().find(|ent| ent.entity.name == name)
	}
}

impl Deref for SceneRef<'_> {
	type Target = SceneData;
	fn deref(&self) -> &Self::Target { self.scene }
}

impl EntityRef<'_> {
	pub fn from<'t>(file: &'t ToyFile, entity: &'t EntityData) -> EntityRef<'t> {
		EntityRef { file, entity }
	}

	pub fn mesh_data(&self) -> Option<&MeshData> {
		let mesh_id = self.entity.mesh_id;

		if mesh_id == 0 {
			return None
		}

		self.file.meshes.get(mesh_id as usize - 1)
	}
}

impl Deref for EntityRef<'_> {
	type Target = EntityData;
	fn deref(&self) -> &Self::Target { self.entity }
}

// TODO: entity queries
// TODO: mesh building