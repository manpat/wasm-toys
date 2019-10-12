#![feature(clamp)]

extern crate wasm_toys as engine;
use engine::prelude::*;


fn main() {
	engine::init_engine(App::new);
}


struct App {
	camera: Camera,
	shader: Shader,

	mesh: DynamicMesh<Vert>,

	base_color: Color,
	waves: Vec<Wave>,
}

impl App {
	fn new() -> App {
		let mut camera = Camera::new();
		camera.set_projection(camera::Projection::Orthographic{ scale: 1.0 });
		camera.set_near_far(-1.0, 1.0);

		let shader = Shader::from_combined(
			include_str!("main.glsl"),
			&["position", "color"]
		);

		App {
			camera,
			shader,

			mesh: DynamicMesh::new(),

			base_color: Color::black(),
			waves: Vec::new(),
		}
	}

	fn generate_waves(&mut self) {
		self.waves.clear();
		let y_offsets = [0.2, -0.1, -0.4];

		let base_hue = rand() * 360.0;
		let hue_delta = rand() * 50.0 - 25.0;
		let hue_variance = hue_delta.abs() / 25.0;

		let value_dir = if rand() > 0.5 { 1.0 } else { -1.0 };
		let value_delta = (rand() * 0.3 + 0.7) * (2.0 - hue_variance.sqrt()*1.5) * 0.1 * value_dir;
		let base_value = rand() * 0.1 + 0.8 - value_delta * 1.5;

		let saturation = 0.9 - (base_value + hue_variance.sqrt() * 0.5) * rand() * 0.3;

		self.base_color = Color::hsv(base_hue, saturation, base_value);

		for (i, &y_off) in y_offsets.iter().enumerate() {
			let i = i as f32 + 1.0;
			let hue = base_hue + hue_delta * i;
			let value = base_value + value_delta * i;

			let target_color = Color::hsv(hue, saturation, value);
			self.waves.push(Wave::new(self.base_color, target_color, y_off));
		}
	}

	fn build_waves(&mut self) {
		let aspect = self.camera.aspect();

		self.mesh.clear();

		for w in self.waves.iter() {
			w.build(&mut self.mesh, aspect, 0.05);
		}
	}
}

impl EngineClient for App {
	fn init(&mut self) {
		self.generate_waves();
	}

	fn update(&mut self, ctx: engine::UpdateContext) {
		if ctx.input.tap() {
			self.generate_waves();
		}

		for wave in self.waves.iter_mut() { wave.update() }

		self.build_waves();

		unsafe {
			let (r,g,b,_) = self.base_color.to_tuple();

			gl::disable(gl::Capability::DepthTest);
			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT);
		}

		self.camera.update(ctx.viewport);

		self.shader.bind();
		self.shader.set_uniform("proj_view", self.camera.projection_view());

		self.mesh.draw(gl::DrawMode::Triangles);
	}
}

#[derive(Debug)]
struct Wave {
	phase: f32,
	freq_mod_phase: f32,
	amp_phase: f32,
	amp_mod_phase: f32,

	freq: f32,
	freq_mod: f32,
	freq_mod_amt: f32,
	amp_freq: f32,
	amp_mod_freq: f32,

	wave_color: Color,
	wave_color_target: Color,
	y_offset: f32,
}

impl Wave {
	fn new(wave_color: Color, wave_color_target: Color, y_offset: f32) -> Self {
		Wave {
			phase: rand().ease_linear(0.0, 2.0 * PI),
			amp_phase: rand().ease_linear(0.0, 2.0 * PI),
			amp_mod_phase: rand().ease_linear(0.0, 2.0 * PI),
			freq_mod_phase: rand().ease_linear(0.0, 2.0 * PI),

			freq: rand().ease_linear(1.0 / 6.0, 1.0 / 3.0) * 2.0 * PI,
			freq_mod: rand().ease_linear(1.0 / 12.0, 1.0 / 6.0) * 2.0 * PI,
			freq_mod_amt: rand().ease_linear(1.0 / 12.0, 1.0 / 6.0) * 2.0 * PI,
			amp_freq: rand().ease_linear(1.0 / 16.0, 1.0 / 9.0) * 2.0 * PI,
			amp_mod_freq: rand().ease_linear(1.0 / 20.0, 1.0 / 8.0) * 2.0 * PI,

			wave_color,
			wave_color_target,
			y_offset
		}
	}

	fn update(&mut self) {
		self.amp_mod_phase += DT * self.amp_mod_freq;
		self.amp_phase += DT * (self.amp_freq * (1.0 + self.amp_mod_phase.sin())) * 0.3;

		self.freq_mod_phase += DT * self.freq_mod;
		self.phase += DT * (self.freq + self.freq_mod_phase.sin() * self.freq_mod_amt) * 0.4;

		self.wave_color = DT.ease_linear(self.wave_color, self.wave_color_target);
	}

	fn build(&self, mb: &mut DynamicMesh<Vert>, aspect: f32, seg_width: f32) {
		let samples = (2.0 * aspect / seg_width).ceil() as usize + 1;
		let mut vs = Vec::with_capacity(samples);

		let wave_color = self.wave_color.into();

		for s in 0..samples {
			let x = s as f32 * seg_width - aspect;

			let amp_mod = (x * self.amp_freq + self.amp_phase).sin() * 0.2;
			let y = (x * self.freq + self.phase).sin() * amp_mod + self.y_offset;

			vs.push( Vert(Vec2::new(x,-1.0), wave_color) );
			vs.push( Vert(Vec2::new(x, y), wave_color) );
		}

		mb.add_tri_strip(&vs);
	}
}


#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vert (Vec2, Vec3);

impl vertex::Vertex for Vert {
	fn descriptor() -> vertex::Descriptor {
		vertex::Descriptor::from(&[2, 3])
	}
}