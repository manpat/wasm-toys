#![feature(box_syntax)]

extern crate wasm_toys as engine;

use common::*;
use engine::graphics::*;
use engine::graphics::vertex::ColorVertex;

fn main() {
	engine::init_engine(SnowApp::new());
}

fn rand() -> f32 {
	unsafe {
		engine::imports::util::math_random()
	}
}

// world space particle size / 2
const PARTICLE_EXTENT: f32 = 1.0 / 20.0;

struct SnowApp {
	camera: Camera,
	particles: Vec<Particle>,

	spawn_timer: f32,

	scene_program: gl::ProgramID,
	snow_program: gl::ProgramID,

	scene_mesh: DynamicMesh<ColorVertex>,
	snow_mesh: BasicDynamicMesh<ParticleVertex>,
}

impl SnowApp {
	fn new() -> Self {
		let mut camera = Camera::new();
		camera.set_near_far(0.1, 100.0);

		let scene_program = create_shader_combined(
			include_str!("scene.glsl"),
			&["position", "color"]
		);

		let snow_program = create_shader_combined(
			include_str!("snow.glsl"),
			&["position", "uv", "lifetime"]
		);

		unsafe {
			gl::enable_attribute(1);
		}

		let snow_mesh = BasicDynamicMesh::new();
		let mut scene_mesh = DynamicMesh::new();

		let ground_color = Vec3::new(1.0, 0.0, 1.0);

		scene_mesh.add_quad(&[
			ColorVertex::new(Vec3::new(-1.0,-1.0, 1.0), ground_color),
			ColorVertex::new(Vec3::new( 1.0,-1.0, 1.0), ground_color),
			ColorVertex::new(Vec3::new( 1.0,-1.0,-1.0), ground_color),
			ColorVertex::new(Vec3::new(-1.0,-1.0,-1.0), ground_color),
		]);

		SnowApp {
			camera,
			particles: Vec::new(),

			spawn_timer: 0.0,

			scene_program, snow_program,
			scene_mesh, snow_mesh,
		}
	}

	fn update_particles(&mut self) {
		self.spawn_timer -= engine::DT;
		if self.spawn_timer < 0.0 {
			let pos_factor = 1.0 - PARTICLE_EXTENT;

			self.particles.push(Particle {
				pos: Vec3::new(
					(rand()*2.0 - 1.0) * pos_factor,
					1.0,
					(rand()*2.0 - 1.0) * pos_factor
				),
				lifetime: 0.0,
				state: ParticleState::Falling,
			});

			self.spawn_timer = rand() * 0.2 + 0.1;
		}

		for p in self.particles.iter_mut() {
			p.lifetime += engine::DT;

			match p.state {
				ParticleState::Falling => {
					let resting_y = -1.0 + PARTICLE_EXTENT;
					p.pos.y -= 0.3 * engine::DT;

					if p.pos.y < resting_y {
						p.pos.y = resting_y;
						p.state = ParticleState::Resting;
						p.lifetime = 0.0;
					}
				}

				ParticleState::Resting => {
					if p.lifetime > 6.0 {
						p.state = ParticleState::Dead;
					}
				}

				ParticleState::Dead => {}
			}
		}

		self.particles.retain(|p| p.state != ParticleState::Dead);
	}

	fn render(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			gl::clear_color(0.2, 0.2, 0.2, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

			gl::disable_attribute(2);

			gl::use_program(self.scene_program);
		}

		let time = ctx.ticks as f32 * engine::DT;

		let quat = Quat::new(Vec3::from_y(1.0), time*0.3);
		let position = quat * Vec3::from_z(3.0) + Vec3::from_y(0.0);

		self.camera.update(ctx.viewport);
		self.camera.set_orientation(quat);
		self.camera.set_position(position);

		gl::set_uniform_mat4(self.scene_program, "u_proj_view", &self.camera.projection_view());
		self.scene_mesh.draw(gl::DrawMode::Triangles);

		unsafe {
			gl::enable_attribute(2);
			gl::use_program(self.snow_program);
		}

		let particle_scale = ctx.viewport.x.min(ctx.viewport.y) as f32 * PARTICLE_EXTENT;
		gl::set_uniform_f32(self.snow_program, "u_particle_scale", particle_scale);
		gl::set_uniform_mat4(self.snow_program, "u_proj_view", &self.camera.projection_view());

		self.snow_mesh.clear();

		for p in self.particles.iter() {
			self.snow_mesh.add_vertex(ParticleVertex {
				pos: p.pos,
				uv: Vec2::zero(),
				lifetime: p.lifetime,
			})
		}

		self.snow_mesh.draw(gl::DrawMode::Points);
	}
}


#[derive(Copy, Clone, PartialEq)]
enum ParticleState {
	Falling,
	Resting,
	Dead,
}

struct Particle {
	pos: Vec3,
	lifetime: f32,
	state: ParticleState,
}


impl engine::EngineClient for SnowApp {
	fn uses_passive_input(&self) -> bool { false }

	fn update(&mut self, ctx: engine::UpdateContext) {
		self.update_particles();
		self.render(ctx);
	}
}


#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct ParticleVertex {
	pos: Vec3,
	uv: Vec2,
	lifetime: f32,
}

impl vertex::Vertex for ParticleVertex {
	fn descriptor() -> vertex::Descriptor {
		vertex::Descriptor::from(&[3, 2, 1])
	}
}