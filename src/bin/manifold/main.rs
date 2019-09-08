extern crate wasm_toys as engine;
use engine::prelude::*;

mod manifold;
use manifold::*;

fn rand() -> f32 {
	unsafe {
		engine::imports::util::math_random()
	}
}

fn main() {
	engine::init_engine(App::new);
}

type Vertex = vertex::ColorVertex;
type Mesh = BasicDynamicMesh<Vertex>;

struct ManifoldObject {
	pos: Vec2,
	color: Vec3,
	vel: Vec2,
}

struct App {
	shader: Shader,
	mesh: Mesh,

	manifold: ToroidManifold,

	position: Vec2, // on the manifold
	velocity: Vec2, // on the manifold
	objects: Vec<ManifoldObject>,
}

impl App {
	fn new() -> App {
		let shader = Shader::from_combined(
			include_str!("main.glsl"),
			&["position", "color"]
		);

		let mut objects = Vec::new();
		for y in -30..30 {
			for x in -30..30 {
				let x = x as f32/30.0 * 2.0/3.0;
				let y = y as f32/30.0 * 2.0;
				let manifold_pos = Vec2::new(x, y);

				objects.push(ManifoldObject {
					pos: manifold_pos,
					color: Vec3::new(0.0, 0.4, 1.0),
					vel: Vec2::zero(),
				});
			}
		}

		objects.push(ManifoldObject {
			pos: Vec2::new(0.0, 0.05),
			color: Vec3::new(1.0, 0.4, 0.0),
			vel: Vec2::zero(),
		});

		App {
			shader,
			mesh: Mesh::new(),
			manifold: ToroidManifold::new(Vec2::splat(10.0)),
			position: Vec2::zero(),
			velocity: Vec2::zero(),
			objects,
		}
	}

	fn rebuild_chart(&mut self) {
		self.mesh.clear();
		self.mesh.add_vertex(Vertex::new(Vec3::zero(), Vec3::splat(1.0)));

		let chart = self.manifold.chart(self.position);
		for obj in self.objects.iter() {
			if let Some(chart_pos) = chart.from_manifold(obj.pos) {
				self.mesh.add_vertex(Vertex::new(chart_pos.extend(0.0), obj.color));
			}
		}
	}
}

impl EngineClient for App {
	fn uses_passive_input(&self) -> bool { false }
	fn drag_threshold(&self) -> Option<u32> { Some(10) }
	fn hold_threshold(&self) -> Option<Ticks> { Some(30) }

	fn update(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			let (r,g,b,_) = Color::hsv(301.0, 0.46, 0.28).to_tuple();

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}

		let chart = self.manifold.chart(self.position);

		if ctx.input.dragging() {
			self.velocity -= ctx.input.frame_delta() * 4.0;
		} else {
			self.velocity *= 1.0 - 4.0*DT;
		}

		self.position += self.velocity * DT;

		if ctx.input.tap() {
			let chart_pos = ctx.input.position();
			let manifold_pos = chart.to_manifold(chart_pos).unwrap();

			for _ in 0..100 {
				self.objects.push(ManifoldObject {
					pos: manifold_pos,
					color: Vec3::new(0.3, 1.0, 0.3),
					vel: Vec2::from_angle(rand() * 2.0 * PI) * rand(),
				});
			}
		}

		for obj in self.objects.iter_mut() {
			let diff = self.manifold.difference(self.position, obj.pos);
			let dist = diff.length().powf(2.0).max(0.05);
			let speed = self.velocity.length();

			obj.vel += diff / dist * DT * 0.1;
			obj.vel += (Vec2::new(rand(), rand()) * 2.0 - 1.0) * DT * 0.002 / dist;
			obj.vel += (self.velocity - obj.vel) / dist.max(1.0) * speed.powi(4).min(1.0) * DT;
			obj.vel *= 1.0 - 0.1*DT;

			if ctx.input.holding() {
				let dist_to_horizon = diff.length().max(0.001) / 5.0;

				obj.vel *= 1.0 - (DT / dist_to_horizon).powf(1.5).min(0.85).max(0.0);
			}

			obj.pos += obj.vel * DT;
		}

		self.rebuild_chart();

		self.shader.bind();
		self.shader.set_uniform("u_proj_view", Mat4::ident());

		self.mesh.draw(gl::DrawMode::Points);
	}
}