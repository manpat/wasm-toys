use engine::prelude::*;
use engine::scene;

use crate::game_state::{GameState, Item};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct SceneView {
	scene_shader: Shader,
	static_mesh: DynamicMesh<SceneVertex>,
	dynamic_mesh: DynamicMesh<SceneVertex>,

	prev_game_state: u64,
}


impl SceneView {
	pub fn new(file: &scene::ToyFile) -> Self {
		let scene_shader = Shader::from_combined(
			include_str!("scene.glsl"),
			&["position", "color"]
		);

		let static_mesh = bake_static_scene_mesh(&file, "main").unwrap();

		SceneView {
			scene_shader,
			static_mesh,
			dynamic_mesh: DynamicMesh::new(),

			prev_game_state: 0,
		}
	}


	pub fn draw(&mut self, proj_view: Mat4, file: &scene::ToyFile, game_state: &GameState) {
		let mut hasher = DefaultHasher::new();
		game_state.hash(&mut hasher); 
		let new_hash = hasher.finish();

		if self.prev_game_state != new_hash {
			self.build_dynamic(file, game_state).unwrap();
			self.prev_game_state = new_hash;
		}

		self.scene_shader.bind();
		self.scene_shader.set_uniform("proj_view", proj_view);

		self.static_mesh.draw(gl::DrawMode::Triangles);
		self.dynamic_mesh.draw(gl::DrawMode::Triangles);
	}


	fn build_dynamic(&mut self, file: &scene::ToyFile, game_state: &GameState) -> EngineResult<()> {
		self.dynamic_mesh.clear();

		if game_state.fishing_hole.fish {
			bake_entity_to_mesh(&mut self.dynamic_mesh, file, find_entity(file, "DYN_FishingHole_Fish")?)?;
		}

		let soup_valid = game_state.soup.is_valid_soup();

		for item in game_state.soup.inventory.iter() {
			let (name, layer) = match item {
				Item::Bucket{ filled: true } => ("DYN_Soup_Base", if soup_valid {"valid"} else {"Col"}),
				Item::Fish{ scaled: true } => ("DYN_Soup_Fish", "scaled"),
				_ => bail!("Invalid item in soup! {:?}", item)
			};

			let entity = find_entity(file, name)?;
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, file, entity, layer)?;
		}

		match game_state.bench.inventory {
			Some(Item::Fish { scaled }) => {
				let col_layer = if scaled { "scaled" } else { "Col" };
				let entity = find_entity(file, "DYN_Bench_Fish")?;
				bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, file, entity, col_layer)?;
			}

			_ => {}
		}

		if game_state.shelf.bucket {
			let entity = find_entity(file, "DYN_Shelf_Bucket")?;
			bake_entity_to_mesh(&mut self.dynamic_mesh, file, entity)?;
		}

		Ok(())
	}
}

pub type SceneVertex = vertex::ColorVertex;

pub fn bake_static_scene_mesh(file: &scene::ToyFile, scene_name: &str) -> EngineResult<DynamicMesh<SceneVertex>> {
	let mut scene_mesh = DynamicMesh::new();

	let ents_with_meshes = entities_in_scene(file, scene_name)
		.filter(|e| e.mesh_id != 0 && !e.name.contains('_'));

	for e in ents_with_meshes {
		bake_entity_to_mesh(&mut scene_mesh, &file, e)?;
	}

	Ok(scene_mesh)
}


pub fn bake_entity_to_mesh<'s>(mesh: &mut DynamicMesh<SceneVertex>, scene: &'s scene::ToyFile, entity: &'s scene::EntityData) -> EngineResult<()> {
	bake_entity_to_mesh_with_color_layer(mesh, scene, entity, "Col")
}


pub fn bake_entity_to_mesh_with_color_layer<'s>(mesh: &mut DynamicMesh<SceneVertex>, scene: &'s scene::ToyFile, entity: &'s scene::EntityData, col: &str) -> EngineResult<()> {
	let mesh_id = entity.mesh_id as usize;

	ensure!(mesh_id != 0, "Entity '{}' has no mesh", entity.name);
	ensure!(mesh_id <= scene.meshes.len(), "Entity '{}' has invalid mesh", entity.name);

	let mesh_data = &scene.meshes[mesh_id-1];

	let color_data = mesh_data.color_data.iter()
		.find(|l| l.name == col)
		.ok_or_else(|| format_err!("Entity '{}'s mesh has no color data layer named '{}'", entity.name, col))?;

	let transform = Mat4::translate(entity.position)
		* entity.rotation.to_mat4()
		* Mat4::scale(entity.scale);

	let verts: Vec<_> = mesh_data.positions.iter()
		.zip(color_data.data.iter())
		.map(|(&pos, col)| {
			vertex::ColorVertex::new(transform * pos, col.to_vec3())
		})
		.collect();

	match mesh_data.indices {
		scene::MeshIndices::U8(ref v) => {
			let indices = v.iter().map(|&i| i as u16);
			mesh.add_geometry(&verts, indices);
		},

		scene::MeshIndices::U16(ref v) => {
			mesh.add_geometry(&verts, v);
		}
	}

	Ok(())
}


pub fn find_entity<'s>(file: &'s scene::ToyFile, name: &str) -> EngineResult<&'s scene::EntityData> {
	file.entities.iter()
		.find(|e| e.name == name)
		.ok_or_else(|| format_err!("Couldn't find entity '{}' in toy file", name))
}


pub fn find_scene<'s>(file: &'s scene::ToyFile, name: &str) -> EngineResult<&'s scene::SceneData> {
	file.scenes.iter()
		.find(|e| e.name == name)
		.ok_or_else(|| format_err!("Couldn't find scene '{}' in toy file", name))
}


pub fn entities_in_scene<'s>(file: &'s scene::ToyFile, scene_name: &str) -> impl Iterator<Item=&'s scene::EntityData> {
	let scene = find_scene(&file, scene_name).unwrap();
	scene.entities.iter()
		.map(move |&id| &file.entities[id as usize - 1])
}