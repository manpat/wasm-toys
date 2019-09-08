#![feature(box_syntax)]

extern crate wasm_toys as engine;
use engine::prelude::*;

fn main() {
	engine::init_engine(VoxelApp::new);
}


struct VoxelApp {
	camera: Camera,
	program: Shader,

	voxel_chunk_mesh: DynamicMesh<Vertex>,
	voxel_chunk_tex: Texture,

	angle_vel: Vec2,
	angle: Vec2,
}

impl VoxelApp {
	fn new() -> VoxelApp {
		let program = Shader::from_combined(
			include_str!("color.glsl"),
			&["position", "normal", "voxel_pos"]
		);

		let mut camera = Camera::new();
		camera.set_near_far(0.5, 5000.0);

		let mut voxel_chunk_tex = TextureBuilder::new()
			.r8()
			.nearest()
			.build();

		let mut voxel_data = [0u8; 8*8*8];

		voxel_data[0*8 + 0] = 1;
		voxel_data[0*8 + 7] = 2;
		voxel_data[7*8 + 0] = 1;
		voxel_data[7*8 + 7] = 2;

		voxel_data[8*8*7 + 0*8 + 0] = 4;
		voxel_data[8*8*7 + 0*8 + 7] = 1;
		voxel_data[8*8*7 + 7*8 + 0] = 4;
		voxel_data[8*8*7 + 7*8 + 7] = 1;

		voxel_data[8*8*3 + 3*8 + 4] = 7;
		voxel_data[8*8*3 + 3*8 + 5] = 5;
		voxel_data[8*8*3 + 4*8 + 4] = 5;
		voxel_data[8*8*3 + 4*8 + 5] = 5;
		
		voxel_data[8*8*4 + 3*8 + 4] = 6;
		voxel_data[8*8*4 + 3*8 + 5] = 7;
		voxel_data[8*8*4 + 4*8 + 4] = 7;
		voxel_data[8*8*4 + 4*8 + 5] = 7;

		voxel_chunk_tex.upload(Vec2i::new(8*8, 8), &voxel_data);

		VoxelApp {
			camera,
			program,

			voxel_chunk_mesh: generate_chunk_mesh(8),
			voxel_chunk_tex,

			angle_vel: Vec2::zero(),
			angle: Vec2::zero(),
		}
	}
}

impl EngineClient for VoxelApp {
	fn uses_passive_input(&self) -> bool { false }

	fn update(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			let (r,g,b,_) = Color::hsv(301.0, 0.46, 0.28).to_tuple();

			gl::enable(gl::Capability::CullFace);

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}

		self.program.bind();

		// spin
		if ctx.input.primary_down() {
			let raw_delta = ctx.input.primary_delta();
			let delta_x = -raw_delta.x as f32 * PI / ctx.viewport.y as f32;
			let delta_y = -raw_delta.y as f32 * PI / ctx.viewport.y as f32;
			self.angle_vel += (Vec2::new(delta_x, delta_y) - self.angle_vel) / 5.0;

		} else {
			self.angle_vel *= 1.0 - 3.0*DT;
		}

		self.angle += self.angle_vel;


		// position camera
		let quat_x = Quat::new(Vec3::from_y(1.0), self.angle.x);
		let quat_y = Quat::new(Vec3::from_x(1.0), self.angle.y);
		let quat = quat_x * quat_y;
		let position = quat * Vec3::from_z(10.0) + Vec3::new(16.0, 4.0, 16.0);

		self.camera.update(ctx.viewport);
		self.camera.set_orientation(quat);
		self.camera.set_position(position);

		self.voxel_chunk_tex.bind(0);

		let proj_view = self.camera.projection_view();

		let chunk_size = 8.0;

		self.program.set_uniform("u_voxel_data", 0);
		self.program.set_uniform("u_voxel_chunk_size", chunk_size);

		for z in 0..8 {
			for x in 0..8 {
				let transform = proj_view * Mat4::translate(Vec3::new(x as f32 * chunk_size, 0.0, z as f32 * chunk_size));

				self.program.set_uniform("u_proj_view", transform);
				self.voxel_chunk_mesh.draw(gl::DrawMode::Triangles);
			}
		}
	}
}


#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
	pub pos: Vec3,
	pub normal: Vec3,
	pub voxel_pos: Vec3,
}

impl vertex::Vertex for Vertex {
	fn descriptor() -> vertex::Descriptor {
		vertex::Descriptor::from(&[3, 3, 3])
	}
}


fn generate_chunk_mesh(chunk_size: i32) -> DynamicMesh<Vertex> {
	let mut mesh = DynamicMesh::new();

	let cube_faces = [
		// west
		[
			Vertex { pos: Vec3::new(0.0, 0.0, 0.0), normal: Vec3::from_x(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(0.0, 0.0, 1.0), normal: Vec3::from_x(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(0.0, 1.0, 1.0), normal: Vec3::from_x(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::from_x(-1.0), voxel_pos: Vec3::zero() },
		],
		// east
		[
			Vertex { pos: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::from_x( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 1.0, 0.0), normal: Vec3::from_x( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 1.0, 1.0), normal: Vec3::from_x( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 0.0, 1.0), normal: Vec3::from_x( 1.0), voxel_pos: Vec3::zero() },
		],
		// north
		[
			Vertex { pos: Vec3::new(0.0, 0.0, 0.0), normal: Vec3::from_z(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::from_z(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 1.0, 0.0), normal: Vec3::from_z(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::from_z(-1.0), voxel_pos: Vec3::zero() },
		],
		// south
		[
			Vertex { pos: Vec3::new(0.0, 0.0, 1.0), normal: Vec3::from_z( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 0.0, 1.0), normal: Vec3::from_z( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 1.0, 1.0), normal: Vec3::from_z( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(0.0, 1.0, 1.0), normal: Vec3::from_z( 1.0), voxel_pos: Vec3::zero() },
		],
		// up
		[
			Vertex { pos: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::from_y( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(0.0, 1.0, 1.0), normal: Vec3::from_y( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 1.0, 1.0), normal: Vec3::from_y( 1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 1.0, 0.0), normal: Vec3::from_y( 1.0), voxel_pos: Vec3::zero() },
		],
		// down
		[
			Vertex { pos: Vec3::new(0.0, 0.0, 0.0), normal: Vec3::from_y(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::from_y(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(1.0, 0.0, 1.0), normal: Vec3::from_y(-1.0), voxel_pos: Vec3::zero() },
			Vertex { pos: Vec3::new(0.0, 0.0, 1.0), normal: Vec3::from_y(-1.0), voxel_pos: Vec3::zero() },
		],
	];


	for z in 0..chunk_size {
		for y in 0..chunk_size {
			for x in 0..chunk_size {
				let offset = Vec3::new(x as f32, y as f32, z as f32);

				for mut face_verts in cube_faces.iter().cloned() {
					for vert in face_verts.iter_mut() {
						vert.pos += offset;
						vert.voxel_pos = offset;
					}

					mesh.add_quad(&face_verts);
				}
			}
		}
	}


	mesh
}