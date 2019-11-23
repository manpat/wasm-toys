extern crate wasm_toys as engine;
use engine::prelude::*;

use vertex::ColorVertex;

mod noise;

const PARTICLE_EXTENT: f32 = 1.0 / 30.0;
const PARTICLE_COUNT: usize = 8000;


struct App {
	camera: Camera,
	camera_focal_point: Vec3,
	camera_dist: f32,

	main_shader: Shader,
	particle_mesh: BasicDynamicMesh<ParticleVertex>,

	color_shader: Shader,
	lines_mesh: BasicDynamicMesh<ColorVertex>,
	grid_lines_mesh: BasicDynamicMesh<ColorVertex>,

	particles: Vec<Particle>,

	perlin: noise::Perlin,
	regen_noise_timer: f32,
}


impl App {
	fn new() -> Self {
		let mut camera = Camera::new();
		camera.set_near_far(0.1, 3000.0);

		let main_shader = Shader::from_combined(
			include_str!("main.glsl"),
			&["position", "part_info"]
		); 

		let color_shader = Shader::from_combined(
			include_str!("color.glsl"),
			&["position", "color"]
		); 

		let particle_mesh = BasicDynamicMesh::new();
		let lines_mesh = BasicDynamicMesh::new();
		let grid_lines_mesh = BasicDynamicMesh::new();

		App {
			camera,
			camera_focal_point: Vec3::zero(),
			camera_dist: 2.0,

			main_shader, color_shader,
			particle_mesh, lines_mesh, grid_lines_mesh,

			particles: Vec::new(),

			perlin: noise::Perlin::new(5),
			regen_noise_timer: rand() * 10.0 + 5.0,
		}
	}

	fn update(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			let (r,g,b,_) = Color::hsv(310.0, 0.3, 0.15).to_tuple();

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}

		self.regen_noise_timer -= DT;

		if ctx.input.tap() || self.regen_noise_timer < 0.0 {
			for _ in 0..100 {
				self.spawn_particle();
			}

			self.perlin = noise::Perlin::new((rand() * 6.0) as usize + 3);
			self.build_lines();

			self.regen_noise_timer = rand() * 10.0 + 5.0;

		} else if ctx.input.holding() {
			for p in self.particles.iter_mut() {
				p.vel += -p.pos * 8.0 * DT;
			}
		}

		self.update_particles();

		let t = ctx.ticks as f32 * DT;

		let part_min = self.particles.iter()
			.map(|p| p.pos)
			.fold(
				Vec3::splat(std::f32::INFINITY),
				|a, p| vec3_map!(a, p.element.min(element))
			);

		let part_max = self.particles.iter()
			.map(|p| p.pos)
			.fold(
				Vec3::splat(std::f32::NEG_INFINITY),
				|a, p| vec3_map!(a, p.element.max(element))
			);

		let part_center = (part_max + part_min) / 2.0;
		let part_spread = (part_max - part_min) / 1.4;

		let max_spread = part_spread.x.max(part_spread.y).max(part_spread.z).max(2.0);

		self.camera_focal_point = (DT/3.0).ease_linear(self.camera_focal_point, part_center);
		self.camera_dist = (DT/3.0).ease_linear(self.camera_dist, max_spread);


		let ori = Quat::new(Vec3::from_y(1.0), t/7.0)
			* Quat::new(Vec3::from_x(1.0), (t/8.0).cos() * -PI/15.0);

		self.camera.update(ctx.viewport);
		self.camera.set_orientation(ori);
		self.camera.set_position(self.camera_focal_point + ori * Vec3::from_z(self.camera_dist));

		let it_size = PARTICLE_EXTENT * ctx.viewport.x.min(ctx.viewport.y) as f32;

		self.main_shader.bind();
		self.main_shader.set_uniform("proj_view", self.camera.projection_view());
		self.main_shader.set_uniform("particle_scale", it_size);

		self.particle_mesh.draw(gl::DrawMode::Points);

		self.color_shader.bind();
		self.color_shader.set_uniform("proj_view", self.camera.projection_view());
		self.grid_lines_mesh.draw(gl::DrawMode::Lines);
		// self.lines_mesh.draw(gl::DrawMode::Lines);
	}

	fn update_particles(&mut self) {
		for p in self.particles.iter_mut() {
			let v = Self::sample(&self.perlin, p.pos);

			p.vel = (8.0*DT).ease_linear(p.vel, v);
			p.pos += (p.vel + rand_vec3() * 0.1) * DT;

			p.lifetime -= DT * (p.pos.length() / 3.0);
		}

		self.particles.retain(|p| p.lifetime > 0.0);

		if self.particles.len() < PARTICLE_COUNT {
			for _ in 0..10 {
				self.spawn_particle();
			}
		}

		self.particle_mesh.clear();

		for p in self.particles.iter() {
			let v = p.vel.extend(p.lifetime);
			self.particle_mesh.add_vertex(ParticleVertex(p.pos, v));
		}
	}

	fn spawn_particle(&mut self) {
		self.particles.push(Particle {
			pos: rand_vec3() * 0.02,
			vel: rand_vec3() * 5.0,
			lifetime: 3.0 + rand() * 50.0,
		});
	}

	fn build_grid(&mut self) {
		self.grid_lines_mesh.clear();

		let grid_color = Color::grey(0.5);
		let bg_color = Color::hsv(310.0, 0.3, 0.15);

		for z in -10..=10 {
			for x in -10..=10 {
				let pos = Vec3::new(x as f32, 0.0, z as f32 + (x % 2) as f32 / 2.0) * 10.0;

				let dt = 1.0 / (1.0 + pos.length()*0.3);

				let color = dt.ease_linear(bg_color, grid_color).to_vec3();

				self.grid_lines_mesh.add_vertices(&[
					ColorVertex::new(pos - Vec3::from_y(100.0), color),
					ColorVertex::new(pos + Vec3::from_y(100.0), color),
				]);
			}
		}
	}

	fn build_lines(&mut self) {
		self.lines_mesh.clear();

		for z in -20..=20 {
			for y in -20..=20 {
				for x in -20..=20 {
					let pos = Vec3::new(x as f32, y as f32, z as f32) / 10.0;
					let vel = Self::sample(&self.perlin, pos);

					let color_b = -vel * 0.1 + 0.5;
					let color_e = vel * 0.5 + 0.5;

					self.lines_mesh.add_vertices(&[
						ColorVertex::new(pos, color_b),
						ColorVertex::new(pos + vel * 0.1, color_e),
					]);
				}
			}
		}
	}

	fn sample(perlin: &noise::Perlin, pos: Vec3) -> Vec3 {
		let v = perlin.gradient(pos);
		// let v = Vec3::new(v.y, v.z, v.x).cross(Vec3::new(v.z, v.x, v.y)) * 2.0;
		let v = Vec3::new(v.y, v.z, v.x)
			+ -pos * 0.01;

		v
	}
}


struct Particle {
	pos: Vec3,
	vel: Vec3,
	lifetime: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ParticleVertex (pub Vec3, pub Vec4);

impl vertex::Vertex for ParticleVertex {
	fn descriptor() -> vertex::Descriptor {
		vertex::Descriptor::from(&[3, 4])
	}
}



impl EngineClient for App {
	fn uses_passive_input(&self) -> bool { true }
	fn captures_input(&self) -> bool { false }
	fn hold_threshold(&self) -> Option<Ticks> { Some(20) }

	fn init(&mut self) {
		for _ in 0..6000 {
			self.spawn_particle();
		}

		self.build_grid();
		self.build_lines();
	}

	fn update(&mut self, ctx: engine::UpdateContext) {
		self.update(ctx);
	}
}

fn main() {
	engine::init_engine(App::new);
}




fn rand_vec3() -> Vec3 {
	Vec3::new(
		rand() * 2.0 - 1.0,
		rand() * 2.0 - 1.0,
		rand() * 2.0 - 1.0,
	)
}