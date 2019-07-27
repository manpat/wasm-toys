#![feature(box_syntax)]

extern crate wasm_toys as engine;

use common::*;
use engine::graphics::*;

fn main() {
	engine::init_engine(SnowApp::new());
}

fn rand() -> f32 {
	unsafe {
		engine::imports::util::math_random()
	}
}

// world space particle size / 2
const PARTICLE_EXTENT: f32 = 1.0 / 30.0;

struct SnowApp {
	camera: Camera,
	particles: Vec<Particle>,

	spawn_timer: f32,
	wind_phase: f32,
	wind_fm_phase: f32,

	scene_program: gl::ProgramID,
	snow_program: gl::ProgramID,

	scene_mesh: BasicDynamicMesh<SceneVertex>,
	snow_mesh: BasicDynamicMesh<ParticleVertex>,
}

impl SnowApp {
	fn new() -> Self {
		let mut camera = Camera::new();
		camera.set_near_far(0.1, 100.0);

		let scene_program = create_shader_combined(
			include_str!("scene.glsl"),
			&["position"]
		);

		let snow_program = create_shader_combined(
			include_str!("snow.glsl"),
			&["position", "sprite_stage"]
		);

		unsafe {
			gl::enable_attribute(1);
		}

		let snow_mesh = BasicDynamicMesh::new();
		let mut scene_mesh = BasicDynamicMesh::new();

		let ground_size = 10.0;

		scene_mesh.add_quad(&[
			SceneVertex(Vec3::new(-ground_size,-1.0, ground_size)),
			SceneVertex(Vec3::new( ground_size,-1.0, ground_size)),
			SceneVertex(Vec3::new( ground_size,-1.0,-ground_size)),
			SceneVertex(Vec3::new(-ground_size,-1.0,-ground_size)),
		]);

		SnowApp {
			camera,
			particles: Vec::new(),

			spawn_timer: 0.0,
			wind_phase: 0.0,
			wind_fm_phase: 0.0,

			scene_program, snow_program,
			scene_mesh, snow_mesh,
		}
	}

	fn update_particles(&mut self) {
		let wind_angle = PI/3.0 + ((self.wind_fm_phase + self.wind_phase)/5.0).sin()*PI/6.0;
		let wind_dir = Vec3::from_y_angle(wind_angle);
		let wind_perp = Vec3::from_y_angle(wind_angle + PI/2.0);

		self.spawn_timer -= engine::DT;
		if self.spawn_timer < 0.0 {
			let spread = 2.0;
			let pos = wind_dir * spread * (rand()*2.0 - 1.0 - 0.2)
				+ wind_perp * spread * (rand()*2.0 - 1.0);

			let pos = Vec3{ y: 2.0, ..pos };

			let spawn_density = (self.wind_fm_phase/17.0).sin();

			self.particles.push(Particle {
				pos,
				lifetime: -rand() * 20.0,
				state: ParticleState::Falling,
			});

			self.spawn_timer = rand() * 0.12 * (1.0 - spawn_density * 0.1);
		}

		self.wind_fm_phase += engine::DT / 5.0;
		let wind_fm = (self.wind_fm_phase + (self.wind_fm_phase/5.0).cos()).sin()*0.8 + 0.1;

		self.wind_phase += engine::DT * (1.0 + wind_fm) * 0.8;

		for p in self.particles.iter_mut() {
			p.lifetime += engine::DT;

			match p.state {
				ParticleState::Falling => {
					let wind_amt = (p.pos.dot(wind_perp) + self.wind_phase*0.8).sin() * 1.8 + 0.2;

					let gravity = Vec3::from_y(-0.1 * (1.0 - wind_amt*0.1));
					let wind = wind_dir * 0.09 * wind_amt;

					p.pos += (gravity + wind) * engine::DT;

					let resting_y = -1.0 + PARTICLE_EXTENT;
					if p.pos.y < resting_y {
						p.pos.y = resting_y;
						p.state = ParticleState::Resting;
						p.lifetime = 0.0;
					}
				}

				ParticleState::Resting => {
					if p.lifetime > 40.0 {
						p.state = ParticleState::Dead;
					}
				}

				ParticleState::Dead => {}
			}
		}

		self.particles.retain(|p| p.state != ParticleState::Dead);

		let cam_pos = self.camera.position();
		let cam_fwd = self.camera.orientation().forward();

		self.particles.sort_by_key(|p| Ordified(-(p.pos - cam_pos).dot(cam_fwd)));
	}

	fn render(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			let (r,g,b,_) = Color::hsv(193.0, 0.15, 0.9).to_tuple();

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

			gl::disable_attribute(1);
			gl::use_program(self.scene_program);
		}

		let time = ctx.ticks as f32 * engine::DT;

		let quat = Quat::new(Vec3::from_y(1.0), (time/3.0).sin()*PI/24.0);
		let position = quat * Vec3::from_z(3.0) + Vec3::from_y(0.0);

		self.camera.update(ctx.viewport);
		self.camera.set_orientation(quat);
		self.camera.set_position(position);

		let ground_color = Color::hsv(105.0, 0.4, 0.8).into();

		gl::set_uniform_vec4(self.scene_program, "u_color", ground_color);
		gl::set_uniform_mat4(self.scene_program, "u_proj_view", &self.camera.projection_view());
		self.scene_mesh.draw(gl::DrawMode::Triangles);

		unsafe {
			gl::enable_attribute(1);
			gl::use_program(self.snow_program);
		}

		let particle_scale = ctx.viewport.y.min(ctx.viewport.x) as f32 * PARTICLE_EXTENT;
		gl::set_uniform_f32(self.snow_program, "u_particle_scale", particle_scale);
		gl::set_uniform_mat4(self.snow_program, "u_proj_view", &self.camera.projection_view());

		self.snow_mesh.clear();

		for p in self.particles.iter() {
			let sprite_stage = match p.state {
				ParticleState::Falling => {
					if p.lifetime < 0.0 {
						0.0
					} else {
						// cycle [1, 2]
						1.0 + p.lifetime/2.0 % 2.0
					}
				}

				ParticleState::Resting => {
					10.0
				}

				ParticleState::Dead => {
					100.0
				}
			};

			self.snow_mesh.add_vertex(ParticleVertex {
				pos: p.pos,
				sprite_stage
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

	fn init(&mut self) {
		// Prebake some particles
		for _ in 0..600 {
			self.update_particles();
		}
	}

	fn update(&mut self, ctx: engine::UpdateContext) {
		self.update_particles();
		self.render(ctx);
	}
}


#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct ParticleVertex {
	pos: Vec3,
	sprite_stage: f32,
}

impl vertex::Vertex for ParticleVertex {
	fn descriptor() -> vertex::Descriptor {
		vertex::Descriptor::from(&[3, 1])
	}
}


#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct SceneVertex (Vec3);

impl vertex::Vertex for SceneVertex {
	fn descriptor() -> vertex::Descriptor {
		vertex::Descriptor::from(&[3])
	}
}