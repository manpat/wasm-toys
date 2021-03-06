extern crate wasm_toys as engine;
use engine::prelude::*;

pub type Mesh = DynamicMesh<vertex::ColorVertex>;


fn main() {
	engine::init_engine(Bubble::new);
}


struct Bubble {
	camera: Camera,
	scene: Mesh,
	portal: Mesh,

	shader: Shader,

	yaw_vel: f32,
	yaw: f32,
}

impl Bubble {
	fn new() -> Bubble {
		let (scene, portal) = init_scene().expect("Error loading scene!");

		let shader = Shader::from_combined(
			include_str!("clipped_color.glsl"),
			&["position", "color"]
		);

		let mut camera = Camera::new();
		camera.set_near_far(0.5, 5000.0);

		Bubble {
			camera,
			scene, portal,
			shader,

			yaw_vel: 0.0,
			yaw: 0.0,
		}
	}
}

impl engine::EngineClient for Bubble {
	fn uses_passive_input(&self) -> bool { false }
	fn drag_threshold(&self) -> Option<u32> { None } // Always drag

	fn update(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			gl::enable(gl::Capability::StencilTest);
			gl::stencil_mask(0xFF);

			let (r,g,b,_) = Color::hsv(301.0, 0.46, 0.28).to_tuple();

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
		}

		self.shader.bind();

		// spin
		if ctx.input.dragging() {
			let raw_delta = ctx.input.frame_delta();
			let delta = -raw_delta.x as f32 * PI * self.camera.aspect();
			self.yaw_vel += (delta - self.yaw_vel) / 5.0;

		} else {
			self.yaw_vel *= 1.0 - 3.0*DT;
		}

		self.yaw += self.yaw_vel;


		// position camera
		let quat = Quat::new(Vec3::from_y(1.0), self.yaw);
		let position = quat * Vec3::from_z(2.0) + Vec3::from_y(2.0);

		self.camera.update(ctx.viewport);
		self.camera.set_orientation(quat);
		self.camera.set_position(position);

		// draw portal mask
		self.shader.set_uniform("proj_view", self.camera.projection_view());
		self.shader.set_uniform("clip_plane", Vec4::new(0.0, 0.0, 0.0,-1.0));

		set_color_write(false);
		set_depth_write(false);
		set_stencil_write(true);
		set_stencil(StencilParams::new(1).always().replace());

		self.portal.draw(gl::DrawMode::Triangles);

		// draw scene - clipped
		self.shader.set_uniform("clip_plane", quat.forward().extend(0.0));

		set_color_write(true);
		set_depth_write(true);
		set_stencil_write(false);
		set_stencil(StencilParams::new(1).equal());

		self.scene.draw(gl::DrawMode::Triangles);

		// TODO bubble shine
		// TODO floaties
	}
}



fn init_scene() -> EngineResult<(Mesh, Mesh)> {
	let file = toy::load(include_bytes!("bubble.toy"))?;

	let mut scene_mesh = Mesh::new();
	let mut portal_mesh = Mesh::new();

	let scene = file.find_scene("seaside")
		.ok_or_else(|| format_err!("Couldn't find scene 'seaside' in toy file"))?;

	let entities = scene.entities.iter()
		.map(|&id| &file.entities[id as usize - 1]);

	for e in entities {
		bake_entity_to_mesh(&mut scene_mesh, &file, e)?;
	}

	let portal_ent = file.find_entity("portal")
		.ok_or_else(|| format_err!("Couldn't find entity 'portal' in toy file"))?;
	bake_entity_to_mesh(&mut portal_mesh, &file, &portal_ent)?;

	scene_mesh.apply(|vert| {
		let rgb = vert.color;

		let (max, min, sep, coeff) = {
			let (max, min, sep, coeff) = if rgb.x > rgb.y {
				(rgb.x, rgb.y, rgb.y - rgb.z, 0.0)
			} else {
				(rgb.y, rgb.x, rgb.z - rgb.x, 2.0)
			};
			
			if rgb.z > max {
				(rgb.z, min, rgb.x - rgb.y, 4.0)
			} else {
				let min_val = if rgb.z < min { rgb.z } else { min };
				(max, min_val, sep, coeff)
			}
		};

		let mut h = 0.0;
		let mut s = 0.0;
		let v = max;

		if max != min {
			let d = max - min;
			s = d / max;
			h = (( sep / d ) + coeff) * 60.0 / 360.0;
		};

		vert.color = Vec3::new(h, s, v);
	});

	Ok((scene_mesh, portal_mesh))
}


fn bake_entity_to_mesh<'s>(mesh: &mut Mesh, scene: &'s toy::Project, entity: &'s toy::EntityData) -> EngineResult<()> {
	let mesh_id = entity.mesh_id as usize;

	ensure!(mesh_id != 0, "Entity '{}' has no mesh", entity.name);
	ensure!(mesh_id <= scene.meshes.len(), "Entity '{}' has invalid mesh", entity.name);

	let mesh_data = &scene.meshes[mesh_id-1];

	ensure!(mesh_data.color_data.len() > 0, "Entity '{}'s mesh has no color data", entity.name);

	let transform = entity.transform();

	let verts: Vec<_> = mesh_data.positions.iter()
		.zip(mesh_data.color_data[0].data.iter())
		.map(|(&pos, col)| {
			vertex::ColorVertex::new(transform * pos, col.to_vec3())
		})
		.collect();

	mesh.add_geometry(&verts, &mesh_data.indices);

	Ok(())
}
