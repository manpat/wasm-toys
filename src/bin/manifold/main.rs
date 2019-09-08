extern crate wasm_toys as engine;

use common::*;
use engine::graphics::*;
use engine::console_log;

mod manifold;
use manifold::*;

fn main() {
	engine::init_engine(App::new);
}

type Vertex = vertex::ColorVertex;
type Mesh = BasicDynamicMesh<Vertex>;

struct ManifoldObject {
	pos: Vec2,
	color: Vec3,
}

struct App {
	shader: Shader,
	mesh: Mesh,

	manifold: ToroidManifold,

	position: Vec2, // on the manifold
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
					color: Vec3::new(0.0, 0.4, 1.0)
				});
			}
		}

		objects.push(ManifoldObject {
			pos: Vec2::new(0.0, 0.05),
			color: Vec3::new(1.0, 0.4, 0.0)
		});

		App {
			shader,
			mesh: Mesh::new(),
			manifold: ToroidManifold::new(Vec2::new(3.0, 2.0)),
			position: Vec2::zero(),
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

impl engine::EngineClient for App {
	fn update(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			let (r,g,b,_) = Color::hsv(301.0, 0.46, 0.28).to_tuple();

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}

		// let delta = self.manifold.chart(self.position).to_manifold(Vec2::new(0.8, 0.3)).unwrap();
		// self.position += delta * engine::DT;

		if ctx.input.button_state(engine::input::MouseButton::Left.into()).is_pressed() {
			// let chart_pos = 
			// console_log(ctx.input.mouse_pos);

		}

		self.rebuild_chart();

		self.shader.bind();
		self.shader.set_uniform("u_proj_view", Mat4::ident());

		self.mesh.draw(gl::DrawMode::Points);
	}
}