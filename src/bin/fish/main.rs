#![feature(box_syntax)]

extern crate wasm_toys as engine;
use engine::prelude::*;
use engine::scene;

mod player_controller;
mod interaction_target;
mod scene_util;
mod game_state;

use player_controller::PlayerController;
use interaction_target::*;
use scene_util::*;
use game_state::*;


fn main() {
	engine::init_engine(App::new);
}

fn rand() -> f32 {
	unsafe {
		engine::imports::util::math_random()
	}
}

// world space particle size / 2
const PARTICLE_EXTENT: f32 = 1.0 / 15.0;



struct App {
	camera: Camera,

	file: scene::ToyFile,

	scene_shader: Shader,
	it_shader: Shader,

	scene_mesh: DynamicMesh<SceneVertex>,
	interaction_target_mesh: BasicDynamicMesh<SceneVertex>,

	player_controller: PlayerController,
}

impl App {
	fn new() -> Self {
		let mut camera = Camera::new();
		camera.set_near_far(0.1, 100.0);

		let scene_shader = Shader::from_combined(
			include_str!("scene.glsl"),
			&["position", "color"]
		);

		let it_shader = Shader::from_combined(
			include_str!("interaction_target.glsl"),
			&["position", "color"]
		);

		let file = scene::load_toy_file(include_bytes!("main.toy")).unwrap();
		let scene_mesh = bake_scene_mesh(&file, "main").unwrap();

		App {
			camera,

			file,

			scene_shader, it_shader,
			scene_mesh,
			interaction_target_mesh: BasicDynamicMesh::new(),

			player_controller: PlayerController::new(),
		}
	}

	fn update(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			let (r,g,b,_) = Color::hsv(193.0, 0.15, 0.9).to_tuple();

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}

		self.camera.update(ctx.viewport);

		self.player_controller.update(&ctx, self.camera.aspect());
		self.player_controller.update_camera(&mut self.camera);

		let static_interaction_targets = interaction_targets_in_range(&self.file, "main", &self.player_controller);

		if ctx.input.tap() {
			if let Some(it) = static_interaction_targets.iter().find(|it| it.suitability == Some(Suitability::Interactible)) {
				console_log!("INTERACTION {}!", it.name);
			}
		}

		// Draw scene
		self.scene_shader.bind();
		self.scene_shader.set_uniform("proj_view", self.camera.projection_view());

		self.scene_mesh.draw(gl::DrawMode::Triangles);

		// Draw interaction targets
		self.it_shader.bind();
		self.it_shader.set_uniform("proj_view", self.camera.projection_view());
		self.it_shader.set_uniform("particle_scale", 60.0);

		self.interaction_target_mesh.clear();
		for it in static_interaction_targets.iter() {
			let color = match it.suitability.unwrap() {
				Suitability::Nearby => Vec3::splat(0.6),
				Suitability::Interactible => Vec3::splat(1.0),
			};

			self.interaction_target_mesh.add_vertex(SceneVertex::new(it.pos, color));
		}

		self.interaction_target_mesh.draw(gl::DrawMode::Points);
	}
}

impl EngineClient for App {
	fn uses_passive_input(&self) -> bool { false }
	fn captures_input(&self) -> bool { true }


	fn init(&mut self) {}

	fn update(&mut self, ctx: engine::UpdateContext) {
		self.update(ctx);
	}
}


