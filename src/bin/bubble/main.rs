#![feature(box_syntax)]

#[macro_use] extern crate wasm_toys as engine;

use common::*;
use engine::DT;
use engine::graphics::*;
use engine::graphics::vertex::ColorVertex as Vertex;

mod scene;

fn main() {
	engine::init_engine(box Bubble::new());
}


struct Bubble {
	camera: Camera,
	scene: DynamicMesh<Vertex>,
	portal: DynamicMesh<Vertex>,

	program: gl::ProgramID,

	yaw_vel: f32,
	yaw: f32,
}

impl Bubble {
	fn new() -> Bubble {
		let (scene, portal) = scene::init();

		let mut shader = include_str!("clipped_color.glsl")
			.split("/* @@@ */");

		let program = create_shader(
			shader.next().unwrap(),
			shader.next().unwrap(),
			&["position", "color"]
		);

		let mut camera = Camera::new();
		camera.set_near_far(0.5, 5000.0);

		Bubble {
			camera,
			scene, portal,
			program,

			yaw_vel: 0.0,
			yaw: 0.0,
		}
	}
}

impl engine::EngineClient for Bubble {
	fn update(&mut self, ctx: engine::UpdateContext) {
		use engine::input;

		unsafe {
			gl::enable(gl::Capability::StencilTest);
			gl::stencil_mask(0xFF);

			let (r,g,b,_) = Color::hsv(301.0, 0.46, 0.28).to_tuple();

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
			gl::use_program(self.program);
		}


		// spin
		if ctx.input.intent_state(input::Intent::Primary).is_down() {
			let delta = -ctx.input.mouse_delta.x as f32 * PI / ctx.viewport.y as f32;
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
		gl::set_uniform_mat4(self.program, "proj_view", &self.camera.projection_view());
		gl::set_uniform_vec4(self.program, "clip_plane", Vec4::new(0.0, 0.0, 0.0,-1.0));

		set_color_write(false);
		set_depth_write(false);
		set_stencil_write(true);
		set_stencil(StencilParams::new(1).always().replace());

		self.portal.draw(gl::DrawMode::Triangles);

		// draw scene - clipped
		gl::set_uniform_vec4(self.program, "clip_plane", quat.forward().extend(0.0));


		set_color_write(true);
		set_depth_write(true);
		set_stencil_write(false);
		set_stencil(StencilParams::new(1).equal());

		self.scene.draw(gl::DrawMode::Triangles);

		// TODO bubble shine
		// TODO floaties
	}
}