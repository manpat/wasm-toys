use engine::prelude::*;
use engine::scene;

pub type SceneVertex = vertex::ColorVertex;

pub fn bake_scene_mesh(file: &scene::ToyFile, scene_name: &str) -> EngineResult<DynamicMesh<SceneVertex>> {
	let mut scene_mesh = DynamicMesh::new();

	let ents_with_meshes = entities_in_scene(file, scene_name)
		.filter(|e| e.mesh_id != 0);

	for e in ents_with_meshes {
		bake_entity_to_mesh(&mut scene_mesh, &file, e)?;
	}

	Ok(scene_mesh)
}


pub fn bake_entity_to_mesh<'s>(mesh: &mut DynamicMesh<SceneVertex>, scene: &'s scene::ToyFile, entity: &'s scene::EntityData) -> EngineResult<()> {
	let mesh_id = entity.mesh_id as usize;

	ensure!(mesh_id != 0, "Entity '{}' has no mesh", entity.name);
	ensure!(mesh_id <= scene.meshes.len(), "Entity '{}' has invalid mesh", entity.name);

	let mesh_data = &scene.meshes[mesh_id-1];

	ensure!(mesh_data.color_data.len() > 0, "Entity '{}'s mesh has no color data", entity.name);

	let transform = Mat4::translate(entity.position)
		* entity.rotation.to_mat4()
		* Mat4::scale(entity.scale);

	let verts: Vec<_> = mesh_data.positions.iter()
		.zip(mesh_data.color_data[0].data.iter())
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