use engine::prelude::*;
use engine::scene;

use crate::game_state::{GameState, Item};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct SceneView {
	scene_shader: Shader,
	static_mesh: DynamicMesh<SceneVertex>,
	dynamic_mesh: DynamicMesh<SceneVertex>,

	ui_mesh: DynamicMesh<SceneVertex>,

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
			ui_mesh: DynamicMesh::new(),

			prev_game_state: 0,
		}
	}


	pub fn draw(&mut self, proj_view: Mat4, file: &scene::ToyFile, game_state: &GameState) {
		let mut hasher = DefaultHasher::new();
		game_state.hash(&mut hasher); 
		let new_hash = hasher.finish();

		if self.prev_game_state != new_hash {
			self.build_dynamic(file, game_state).unwrap();
			self.build_ui(file, game_state).unwrap();
			self.prev_game_state = new_hash;
		}

		self.scene_shader.bind();
		self.scene_shader.set_uniform("proj_view", proj_view);

		self.static_mesh.draw(gl::DrawMode::Triangles);
		self.dynamic_mesh.draw(gl::DrawMode::Triangles);
	}


	pub fn draw_ui(&mut self, proj_view: Mat4) {
		self.scene_shader.bind();
		self.scene_shader.set_uniform("proj_view", proj_view);
		self.ui_mesh.draw(gl::DrawMode::Triangles);
	}


	fn build_dynamic(&mut self, file: &scene::ToyFile, game_state: &GameState) -> EngineResult<()> {
		self.dynamic_mesh.clear();

		console_log!("{:#?}", game_state);

		let soup_valid = game_state.cauldron.is_valid_soup();

		for item in game_state.cauldron.inventory.iter() {
			let (name, layer): (_, &str) = match item {
				Item::Bucket{ filled: true } => ("DYN_Soup_Base", if soup_valid {"broth"} else {"water"}),
				Item::Bucket{ filled: false } => ("DYN_Soup_Bucket", "Col"),
				Item::Fish{ variant } => ("DYN_Soup_Fish", &variant),
				_ => bail!("Invalid item in soup! {:?}", item)
			};

			let entity = find_entity(file, name)?;
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, file, entity, layer)?;
		}

		match &game_state.bench.inventory {
			Some(Item::Fish { variant }) => {
				let entity = find_entity(file, "DYN_Bench_Fish")?;
				bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, file, entity, &variant)?;
			}

			_ => {}
		}

		if let Some(Item::Bucket{ filled }) = game_state.shelf.inventory {
			let name = if filled { "DYN_Shelf_Bucket_Filled" } else { "DYN_Shelf_Bucket" };
			let entity = find_entity(file, name)?;
			bake_entity_to_mesh(&mut self.dynamic_mesh, file, entity)?;
		}

		if game_state.market.red_fish {
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, file, find_entity(file, "DYN_Market_Fish_Red")?, "red")?;
		}

		if game_state.market.green_fish {
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, file, find_entity(file, "DYN_Market_Fish_Green")?, "green")?;
		}

		if game_state.market.orange_fish {
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, file, find_entity(file, "DYN_Market_Fish_Orange")?, "orange")?;
		}

		if game_state.market.blue_fish {
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, file, find_entity(file, "DYN_Market_Fish_Blue")?, "blue")?;
		}

		Ok(())
	}

	fn build_ui(&mut self, file: &scene::ToyFile, game_state: &GameState) -> EngineResult<()> {
		self.ui_mesh.clear();

		if game_state.inventory.is_none() {
			return Ok(())
		}

		match game_state.inventory.as_ref().unwrap() {
			Item::Bucket{ filled } => if *filled {
				bake_entity_with_new_origin(&mut self.ui_mesh, file, find_entity(file, "BucketFilled")?, "Col", None)
			} else {
				bake_entity_with_new_origin(&mut self.ui_mesh, file, find_entity(file, "Bucket")?, "Col", None)
			}

			Item::Fish{ variant } => bake_entity_with_new_origin(&mut self.ui_mesh, file, find_entity(file, "Fish")?, variant, None),
			Item::Coin => bake_entity_with_new_origin(&mut self.ui_mesh, file, find_entity(file, "Coin")?, "Col", None),
				
			Item::Soup(ingredients) => {
				let soup = find_entity(file, "Soup")?;
				bake_entity_with_new_origin(&mut self.ui_mesh, file, soup, "Col", Some(soup.position))?;

				for item in ingredients {
					let (ent_name, layer) = match item {
						Item::Fish{..} => ("Soup_Fish", "scaled"),
						_ => continue
					};

					let entity = find_entity(file, ent_name)?;
					bake_entity_with_new_origin(&mut self.ui_mesh, file, entity, layer, Some(soup.position))?;
				}

				Ok(())
			}

			_ => Ok(())
		}
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
	bake_entity_with_new_origin(mesh, scene, entity, col, Some(Vec3::zero()))
}


pub fn bake_entity_with_new_origin<'s>(mesh: &mut DynamicMesh<SceneVertex>, scene: &'s scene::ToyFile, entity: &'s scene::EntityData, col: &str, origin: Option<Vec3>) -> EngineResult<()> {
	let mesh_id = entity.mesh_id as usize;

	ensure!(mesh_id != 0, "Entity '{}' has no mesh", entity.name);
	ensure!(mesh_id <= scene.meshes.len(), "Entity '{}' has invalid mesh", entity.name);

	let mesh_data = &scene.meshes[mesh_id-1];

	let color_data = mesh_data.color_data.iter()
		.find(|l| l.name == col)
		.ok_or_else(|| format_err!("Entity '{}'s mesh has no color data layer named '{}'", entity.name, col))?;

	let origin = origin.unwrap_or(entity.position);
	let transform = Mat4::translate(entity.position - origin)
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