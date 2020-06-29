use engine::prelude::*;

use crate::game_state::{GameState, Item};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub type SceneMesh = DynamicMesh<SceneVertex>;


pub struct SceneView {
	scene_shader: Shader,
	static_mesh: SceneMesh,
	dynamic_mesh: SceneMesh,

	ui_mesh: SceneMesh,

	prev_game_state: u64,
}


impl SceneView {
	pub fn new(file: &toy::Project) -> Self {
		let scene_shader = Shader::from_combined(
			include_str!("scene.glsl"),
			&["position", "color"]
		);

		let main_scene = file.find_scene("main").unwrap();
		let static_mesh = bake_static_scene_mesh(main_scene).unwrap();

		SceneView {
			scene_shader,
			static_mesh,
			dynamic_mesh: DynamicMesh::new(),
			ui_mesh: DynamicMesh::new(),

			prev_game_state: 0,
		}
	}

	pub fn update(&mut self, file: &toy::Project, game_state: &GameState) {
		let mut hasher = DefaultHasher::new();
		game_state.hash(&mut hasher); 
		let new_hash = hasher.finish();

		if self.prev_game_state != new_hash {
			self.build_dynamic(file, game_state).unwrap();
			self.build_ui(file, game_state).unwrap();
			self.prev_game_state = new_hash;
		}
	}


	pub fn draw(&mut self, proj_view: Mat4) {
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


	fn build_dynamic(&mut self, file: &toy::Project, game_state: &GameState) -> EngineResult<()> {
		self.dynamic_mesh.clear();

		// draw cauldron
		let soup_valid = game_state.cauldron.is_valid_soup();

		for item in game_state.cauldron.inventory.iter() {
			let (name, layer): (_, &str) = match item {
				Item::Bucket{ filled: true } => ("DYN_Soup_Base", if soup_valid {"broth"} else {"water"}),
				Item::Bucket{ filled: false } => ("DYN_Soup_Bucket", toy::DEFAULT_COLOR_DATA_NAME),
				Item::Fish{ variant } => ("DYN_Soup_Fish", &variant),
				_ => bail!("Invalid item in soup! {:?}", item)
			};

			let entity = find_entity(file, name)?;
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, entity, layer)?;
		}

		// draw bench
		match &game_state.bench.inventory {
			Some(Item::Fish { variant }) => {
				let entity = find_entity(file, "DYN_Bench_Fish")?;
				bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, entity, &variant)?;
			}

			_ => {}
		}

		bake_entity_to_mesh(&mut self.dynamic_mesh, find_entity(file, "DYN_Rope_Garlic")?)?;

		// draw shelf
		if let Some(Item::Bucket{ filled }) = game_state.shelf.inventory {
			let name = if filled { "DYN_Shelf_Bucket_Filled" } else { "DYN_Shelf_Bucket" };
			let entity = find_entity(file, name)?;
			bake_entity_to_mesh(&mut self.dynamic_mesh, entity)?;
		}

		// draw fishing hole
		if game_state.fishing_hole.red_fish {
			let entity = find_entity(file, "DYN_Market_Fish_Red")?;
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, entity, "red")?;
		}

		if game_state.fishing_hole.green_fish {
			let entity = find_entity(file, "DYN_Market_Fish_Green")?;
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, entity, "green")?;
		}

		if game_state.fishing_hole.orange_fish {
			let entity = find_entity(file, "DYN_Market_Fish_Orange")?;
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, entity, "orange")?;
		}

		if game_state.fishing_hole.blue_fish {
			let entity = find_entity(file, "DYN_Market_Fish_Blue")?;
			bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, entity, "blue")?;
		}

		// draw table
		let stack_dist = 0.15;
		for (i, item) in game_state.table.inventory.iter().enumerate() {
			use std::ops::Deref;
			match item {
				Item::Soup(ingredients) => {
					let mut soup_entity = find_entity(file, "DYN_Table_Soup")?.deref().clone();
					soup_entity.position.y += i as f32 * stack_dist;
					bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, toy::EntityRef::from(file, &soup_entity), toy::DEFAULT_COLOR_DATA_NAME)?;

					for item in ingredients {
						let (ent_name, layer) = match item {
							Item::Fish{..} => ("DYN_Table_Soup_Fish", "scaled"),
							_ => continue
						};

						let mut entity = find_entity(file, ent_name)?.deref().clone();
						entity.position.y += i as f32 * stack_dist;
						bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, toy::EntityRef::from(file, &entity), layer)?;
					}

				}

				Item::EmptyBowl => {
					let mut entity = find_entity(file, "DYN_Table_EmptyBowl")?.deref().clone();
					entity.position.y += i as f32 * stack_dist;
					bake_entity_to_mesh_with_color_layer(&mut self.dynamic_mesh, toy::EntityRef::from(file, &entity), toy::DEFAULT_COLOR_DATA_NAME)?;
				}

				_ => {}
			}
		}

		Ok(())
	}

	fn build_ui(&mut self, file: &toy::Project, game_state: &GameState) -> EngineResult<()> {
		self.ui_mesh.clear();

		if game_state.inventory.is_none() {
			return Ok(())
		}

		match game_state.inventory.as_ref().unwrap() {
			Item::Bucket{ filled } => if *filled {
				bake_entity_with_new_origin(&mut self.ui_mesh, find_entity(file, "BucketFilled")?, toy::DEFAULT_COLOR_DATA_NAME, None)
			} else {
				bake_entity_with_new_origin(&mut self.ui_mesh, find_entity(file, "Bucket")?, toy::DEFAULT_COLOR_DATA_NAME, None)
			}

			Item::Fish{ variant } => bake_entity_with_new_origin(&mut self.ui_mesh, find_entity(file, "Fish")?, variant, None),
				
			Item::Soup(ingredients) => {
				let soup = find_entity(file, "Soup")?;
				bake_entity_with_new_origin(&mut self.ui_mesh, soup, toy::DEFAULT_COLOR_DATA_NAME, Some(soup.position))?;

				for item in ingredients {
					let (ent_name, layer) = match item {
						Item::Fish{..} => ("Soup_Fish", "scaled"),
						_ => continue
					};

					let entity = find_entity(file, ent_name)?;
					bake_entity_with_new_origin(&mut self.ui_mesh, entity, layer, Some(soup.position))?;
				}

				Ok(())
			}

			_ => Ok(())
		}
	}
}

pub type SceneVertex = vertex::ColorVertex;

pub fn bake_static_scene_mesh(scene: toy::SceneRef) -> EngineResult<SceneMesh> {
	let mut scene_mesh = DynamicMesh::new();

	let ents_with_meshes = scene.entities()
		.filter(|e| e.mesh_id != 0 && !e.name.contains('_'));

	for e in ents_with_meshes {
		bake_entity_to_mesh(&mut scene_mesh, e)?;
	}

	Ok(scene_mesh)
}


pub fn bake_entity_to_mesh<'s>(mesh: &mut SceneMesh, entity: toy::EntityRef) -> EngineResult<()> {
	bake_entity_with_new_origin(mesh, entity, toy::DEFAULT_COLOR_DATA_NAME, Some(Vec3::zero()))
}


pub fn bake_entity_to_mesh_with_color_layer<'s>(mesh: &mut SceneMesh, entity: toy::EntityRef, col: &str) -> EngineResult<()> {
	bake_entity_with_new_origin(mesh, entity, col, Some(Vec3::zero()))
}


pub fn bake_entity_with_new_origin(mesh: &mut SceneMesh, entity: toy::EntityRef, col: &str, origin: Option<Vec3>) -> EngineResult<()> {
	let mesh_data = entity.mesh_data()
		.ok_or_else(|| format_err!("Entity '{}' has no mesh", entity.name))?;

	let color_data = mesh_data.color_data(col)
		.ok_or_else(|| format_err!("Entity '{}'s mesh has no color data layer named '{}'", entity.name, col))?;

	let origin = origin.unwrap_or(entity.position);
	let transform = Mat4::translate(entity.position - origin)
		* entity.rotation.to_mat4()
		* Mat4::scale(entity.scale);

	bake_mesh_with_transform(mesh, mesh_data, color_data, transform);

	Ok(())
}


fn bake_mesh_with_transform(mesh: &mut SceneMesh, mesh_data: &toy::MeshData, color_data: &toy::MeshColorData, transform: Mat4) {
	let verts: Vec<_> = mesh_data.positions.iter()
		.zip(color_data.data.iter())
		.map(|(&pos, col)| {
			vertex::ColorVertex::new(transform * pos, col.to_vec3())
		})
		.collect();

	match mesh_data.indices {
		toy::MeshIndices::U8(ref v) => {
			let indices = v.iter().map(|&i| i as u16);
			mesh.add_geometry(&verts, indices);
		},

		toy::MeshIndices::U16(ref v) => {
			mesh.add_geometry(&verts, v);
		}
	}
}


pub fn find_entity<'toy>(file: &'toy toy::Project, name: &str) -> EngineResult<toy::EntityRef<'toy>> {
	file.find_entity(name)
		.ok_or_else(|| format_err!("Couldn't find entity '{}' in toy file", name))
}
