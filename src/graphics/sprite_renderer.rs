use crate::graphics::camera::Camera;
use crate::graphics::mesh::DynamicMesh;
use crate::graphics::vertex::TexturedVertex;
use crate::graphics::texture_registry::{TextureRegistry, NULL_TEXTURE};
use crate::graphics::sprite_registry::AnimationFrame;
use crate::imports::gl::{self, DrawMode};
use common::math::*;

#[derive(Copy, Clone, Debug)]
pub enum SpriteOrientation {
	UprightStatic{heading: f32},
	Flat{heading: f32},
	Billboard,
	UprightBillboard,
}

struct SpriteInstance {
	pos: Vec3,
	px_scale: f32,
	frame: AnimationFrame,
	orientation: SpriteOrientation,
}

pub struct SpriteRenderer {
	instances: Vec<SpriteInstance>,
	mesh: DynamicMesh<TexturedVertex>,
}

impl SpriteRenderer {
	pub fn new() -> Self {
		SpriteRenderer {
			instances: Vec::new(),
			mesh: DynamicMesh::new(),
		}
	}

	pub fn add_upright_billboard(&mut self, pos: Vec3, px_scale: f32, frame: AnimationFrame) {
		let orientation = SpriteOrientation::UprightBillboard;
		self.instances.push(SpriteInstance{pos, px_scale, frame, orientation});
	}

	pub fn add_billboard(&mut self, pos: Vec3, px_scale: f32, frame: AnimationFrame) {
		let orientation = SpriteOrientation::Billboard;
		self.instances.push(SpriteInstance{pos, px_scale, frame, orientation});
	}

	pub fn add_instance(&mut self, pos: Vec3, px_scale: f32, frame: AnimationFrame, orientation: SpriteOrientation) {
		self.instances.push(SpriteInstance{pos, px_scale, frame, orientation});
	}

	pub fn add_2d_instance(&mut self, pos: Vec2, px_scale: f32, frame: AnimationFrame) {
		self.instances.push(SpriteInstance{
			pos: pos.extend(0.0),
			px_scale, frame,
			orientation: SpriteOrientation::Flat{heading: 0.0}
		});
	}

	pub fn draw_batched_world(&mut self, camera: &Camera, texture_registry: &TextureRegistry) {
		use self::SpriteOrientation::*;

		let ori = camera.orientation();
		
		let cam_right = ori.right();
		let cam_up = ori.up();

		// TODO: This probably won't work very well in other projections
		// but it's fine for first person + perspective
		let xz_right = (cam_right * Vec3::new(1.0, 0.0, 1.0)).normalize();

		self.instances.sort_unstable_by_key(|i| i.frame.texture);
		self.instances.retain(|i| i.frame.texture != NULL_TEXTURE);

		let mut prev_tex_id = NULL_TEXTURE;
		let mut tex_size = Vec2::zero();

		for SpriteInstance{pos, px_scale, frame, orientation} in self.instances.iter() {
			if prev_tex_id != frame.texture {
				self.mesh.draw(DrawMode::Triangles);
				self.mesh.clear();

				let tex_info = texture_registry.get_texture_info(frame.texture).unwrap();

				unsafe { gl::bind_texture(tex_info.gl_id); }
				prev_tex_id = frame.texture;
				tex_size = tex_info.size.to_vec2();
			}

			let frame_size = frame.size.to_vec2();

			// TODO: move origin
			let w = px_scale * frame_size.x / 2.0;
			let h = px_scale * frame_size.y;

			let uv_start = frame.pos.to_vec2() / tex_size;
			let Vec2{ x: uvw, y: uvh } = frame_size / tex_size;

			let up = match orientation {
				UprightStatic{..} => Vec3::from_y(1.0),
				Flat{heading} => Vec3::from_y_angle(*heading - PI/2.0),
				UprightBillboard => Vec3::from_y(1.0),
				Billboard => cam_up,
			};

			let right = match orientation {
				UprightStatic{heading} => Vec3::from_y_angle(*heading),
				Flat{heading} => Vec3::from_y_angle(*heading),
				UprightBillboard => xz_right,
				Billboard => xz_right
			};

			let centered = match orientation {
				Flat{..} => true,
				_ => false,
			};

			let m = 0.001;

			let vs = if centered {
				let h = h / 2.0;

				[
					TexturedVertex::new(*pos - right*w - up*h, 	uv_start + Vec2::new(m, uvh-m)),
					TexturedVertex::new(*pos - right*w + up*h, 	uv_start + Vec2::new(m, m)),
					TexturedVertex::new(*pos + right*w + up*h, 	uv_start + Vec2::new(uvw-m, m)),
					TexturedVertex::new(*pos + right*w - up*h, 	uv_start + Vec2::new(uvw-m, uvh-m)),
				]

			} else {
				[
					TexturedVertex::new(*pos - right*w, 		uv_start + Vec2::new(m, uvh-m)),
					TexturedVertex::new(*pos - right*w + up*h, 	uv_start + Vec2::new(m, m)),
					TexturedVertex::new(*pos + right*w + up*h, 	uv_start + Vec2::new(uvw-m, m)),
					TexturedVertex::new(*pos + right*w, 		uv_start + Vec2::new(uvw-m, uvh-m)),
				]
			};

			self.mesh.add_quad(&vs);
		}

		self.mesh.draw(DrawMode::Triangles);
		self.mesh.clear();
		self.instances.clear();
	}


	pub fn draw_batched_flat(&mut self, texture_registry: &TextureRegistry) {
		self.instances.sort_unstable_by_key(|i| i.frame.texture);
		self.instances.retain(|i| i.frame.texture != NULL_TEXTURE);

		let mut prev_tex_id = NULL_TEXTURE;
		let mut tex_size = Vec2::zero();

		for SpriteInstance{pos, px_scale, frame, ..} in self.instances.iter() {
			if prev_tex_id != frame.texture {
				self.mesh.draw(DrawMode::Triangles);
				self.mesh.clear();

				let tex_info = texture_registry.get_texture_info(frame.texture).unwrap();

				unsafe { gl::bind_texture(tex_info.gl_id); }
				prev_tex_id = frame.texture;
				tex_size = tex_info.size.to_vec2();
			}

			let frame_size = frame.size.to_vec2();

			let uv_start = frame.pos.to_vec2() / tex_size;
			let Vec2{ x: uvw, y: uvh } = frame_size / tex_size;

			let m = 0.001;

			let w = px_scale * frame_size.x;
			let h = px_scale * frame_size.y;

			let rt = Vec3::from_x(w/2.0);
			let up = Vec3::from_y(h/2.0);

			let vs = [
				TexturedVertex::new(*pos - rt - up, 	uv_start + Vec2::new(m, uvh-m)),
				TexturedVertex::new(*pos - rt + up, 	uv_start + Vec2::new(m, m)),
				TexturedVertex::new(*pos + rt + up, 	uv_start + Vec2::new(uvw-m, m)),
				TexturedVertex::new(*pos + rt - up, 	uv_start + Vec2::new(uvw-m, uvh-m)),
			];

			self.mesh.add_quad(&vs);
		}

		self.mesh.draw(DrawMode::Triangles);
		self.mesh.clear();
		self.instances.clear();
	}


	pub fn draw_single_ortho(&mut self, texture_registry: &TextureRegistry, px_scale: f32, frame: AnimationFrame) {
		let tex_info = texture_registry.get_texture_info(frame.texture).unwrap();
		unsafe { gl::bind_texture(tex_info.gl_id); }

		let tex_size = tex_info.size.to_vec2();
		let frame_size = frame.size.to_vec2();

		let uv_start = frame.pos.to_vec2() / tex_size;
		let Vec2{ x: uvw, y: uvh } = frame_size / tex_size;

		let w = px_scale * frame_size.x;
		let h = px_scale * frame_size.y;

		let rt = Vec3::from_x(w/2.0);
		let up = Vec3::from_y(h/2.0);

		let m = 0.001;

		let vs = [
			TexturedVertex::new(-rt - up, 	uv_start + Vec2::new(m, uvh-m)),
			TexturedVertex::new(-rt + up, 	uv_start + Vec2::new(m, m)),
			TexturedVertex::new( rt + up, 	uv_start + Vec2::new(uvw-m, m)),
			TexturedVertex::new( rt - up, 	uv_start + Vec2::new(uvw-m, uvh-m)),
		];

		self.mesh.add_quad(&vs);
		self.mesh.draw(DrawMode::Triangles);
		self.mesh.clear();
	}
}