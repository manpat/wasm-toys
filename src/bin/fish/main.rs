#![feature(box_syntax)]
#![feature(clamp)]

extern crate wasm_toys as engine;
use engine::prelude::*;
use engine::scene;

use vertex::BasicVertex;

mod player_controller;
mod interaction_target;
mod scene_view;
mod game_state;

use player_controller::PlayerController;
use interaction_target::*;
use scene_view::*;
use game_state::*;


fn main() {
	engine::init_engine(App::new);
}

// world space particle size / 2
const PARTICLE_EXTENT: f32 = 1.0 / 15.0;


#[derive(Debug, Copy, Clone)]
enum PlayState {
	Normal,
	EnterSleep(f32),
	Sleeping(f32),
	LeaveSleep(f32),
}

struct App {
	camera: Camera,

	file: scene::ToyFile,

	it_shader: Shader,
	interaction_target_mesh: BasicDynamicMesh<SceneVertex>,

	scene_view: SceneView,
	game_state: GameState,

	player_controller: PlayerController,

	screen_transition_shader: Shader,
	screen_transition_mesh: BasicDynamicMesh<BasicVertex>,
	play_state: PlayState,
}

impl App {
	fn new() -> Self {
		let mut camera = Camera::new();
		camera.set_near_far(0.1, 3000.0);

		let it_shader = Shader::from_combined(
			include_str!("interaction_target.glsl"),
			&["position", "color"]
		); 

		let screen_transition_shader = Shader::from_combined(
			include_str!("transition.glsl"),
			&["position"]
		);

		let mut screen_transition_mesh = BasicDynamicMesh::new();
		screen_transition_mesh.add_quad(&[
			BasicVertex(Vec3::new(-1.0, -1.0, 0.0)),
			BasicVertex(Vec3::new(-1.0,  1.0, 0.0)),
			BasicVertex(Vec3::new( 1.0,  1.0, 0.0)),
			BasicVertex(Vec3::new( 1.0, -1.0, 0.0)),
		]);

		let file = scene::load_toy_file(include_bytes!("main.toy")).unwrap();
		let scene_view = SceneView::new(&file);

		App {
			camera,

			file,

			it_shader,
			interaction_target_mesh: BasicDynamicMesh::new(),

			scene_view,
			game_state: GameState::new(),

			player_controller: PlayerController::new(),

			screen_transition_shader,
			screen_transition_mesh,
			play_state: PlayState::LeaveSleep(0.0),
		}
	}

	fn update(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			let (r,g,b,_) = Color::hsv(193.0, 0.15, 0.9).to_tuple();

			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}

		if ctx.input_raw.button_state(engine::input::KeyCode::F1).is_pressed() {
			self.player_controller.toggle_cheat_hacker_mode();
		}

		self.camera.update(ctx.viewport);

		let main_scene = self.file.find_scene("main").unwrap();
		let static_interaction_targets = interaction_targets_in_range(main_scene, &self.player_controller);

		match self.play_state {
			PlayState::Normal => {
				self.player_controller.update(&ctx, self.camera.aspect());
				self.player_controller.update_camera(&mut self.camera);

				if ctx.input.tap() {
					if let Some(it) = static_interaction_targets.iter().find(|it| it.suitability == Some(Suitability::Interactible)) {
						self.game_state.interact(&it.name);

						if self.game_state.in_bed {
							self.play_state = PlayState::EnterSleep(0.0);
						}
					}
				}
			}

			PlayState::EnterSleep(t) => {
				if t > 1.0 {
					self.game_state = GameState::new();
					self.play_state = PlayState::Sleeping(0.0);
				} else {
					self.play_state = PlayState::EnterSleep(t + DT);
				}
			}

			PlayState::Sleeping(t) => {
				if t > 1.0 {
					self.play_state = PlayState::LeaveSleep(0.0);
				} else {
					self.play_state = PlayState::Sleeping(t + 1.5*DT);
				}
			}

			PlayState::LeaveSleep(t) => {
				if t > 1.0 {
					self.play_state = PlayState::Normal;
				} else {
					self.play_state = PlayState::LeaveSleep(t + 0.8*DT);
				}
			}
		}

		// Draw scene
		self.scene_view.draw(self.camera.projection_view(), &self.file, &self.game_state);

		// Draw interaction targets
		let it_size = PARTICLE_EXTENT * ctx.viewport.x.min(ctx.viewport.y) as f32;

		self.it_shader.bind();
		self.it_shader.set_uniform("proj_view", self.camera.projection_view());
		self.it_shader.set_uniform("particle_scale", it_size);

		self.interaction_target_mesh.clear();
		for it in static_interaction_targets.iter() {
			if self.game_state.can_interact(&it.name) {
				let color = match it.suitability.unwrap() {
					Suitability::Nearby => Vec3::splat(0.6),
					Suitability::Interactible => Vec3::splat(1.0),
				};

				self.interaction_target_mesh.add_vertex(SceneVertex::new(it.pos, color));
			}
		}

		self.interaction_target_mesh.draw(gl::DrawMode::Points);

		// Draw UI
		unsafe {
			gl::clear(gl::DEPTH_BUFFER_BIT);
		}

		let time = ctx.ticks as f32 * engine::DT;

		let ui_transform = self.camera.projection_matrix()
			* Mat4::translate(Vec3::new(0.0, -0.3, -1.0))
			* Mat4::scale(Vec3::splat(0.3))
			* Mat4::xrot(PI/8.0)
			* Mat4::yrot(time);

		self.scene_view.draw_ui(ui_transform);

		// Draw screen fade
		unsafe {
			gl::clear(gl::DEPTH_BUFFER_BIT);
		}

		let aspect = self.camera.aspect();
		let max_fade = -2.0 * aspect;

		self.screen_transition_shader.bind();
		self.screen_transition_shader.set_uniform("aspect", aspect);
		self.screen_transition_shader.set_uniform("fade_color", Color::hsv(301.0, 0.46, 0.28).to_vec4());

		match self.play_state {
			PlayState::Normal => {
				self.screen_transition_shader.set_uniform("fade_amount", max_fade);
			}

			PlayState::EnterSleep(t) => {
				let amt = t.ease_exp_inout(max_fade, 1.0);
				self.screen_transition_shader.set_uniform("fade_amount", amt);
			}

			PlayState::Sleeping(_) => {
				self.screen_transition_shader.set_uniform("fade_amount", 1.0);
			}

			PlayState::LeaveSleep(t) => {
				let amt = t.ease_exp_in(1.0, max_fade);
				self.screen_transition_shader.set_uniform("fade_amount", amt);
			}
		}

		self.screen_transition_mesh.draw(gl::DrawMode::Triangles);
	}
}

impl EngineClient for App {
	fn uses_passive_input(&self) -> bool { false }
	fn captures_input(&self) -> bool { true }


	fn init(&mut self) {
		self.player_controller.update_camera(&mut self.camera);
	}

	fn update(&mut self, ctx: engine::UpdateContext) {
		self.update(ctx);
	}
}


