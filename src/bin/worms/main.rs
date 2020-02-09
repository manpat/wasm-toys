#![feature(clamp)]

extern crate wasm_toys as engine;
use engine::prelude::*;

use std::iter;


fn main() {
	engine::init_engine(App::new);
}


struct App {
	camera: Camera,
	shader: Shader,
	mesh: DynamicMesh<Vert>,

	worms: Vec<Worm>,
}


const WORLD_SCALE: f32 = 5.0;


impl App {
	fn new() -> App {
		let mut camera = Camera::new();
		camera.set_projection(camera::Projection::Orthographic{ scale: WORLD_SCALE });
		camera.set_near_far(-1.0, 1.0);

		let shader = Shader::from_combined(
			include_str!("main.glsl"),
			&["pos_part_a", "pos_b", "color", "side", "body_pos", "width"]
		);

		App {
			camera,
			shader,
			mesh: DynamicMesh::new(),

			worms: Vec::new(),
		}
	}
}


impl EngineClient for App {
	fn init(&mut self) {
		for _ in 0..20 {
			let worm_pos = Vec2::new(rand(), rand()) * 2.0 - 1.0;
			self.worms.push(Worm::new(worm_pos * WORLD_SCALE));
		}
	}

	fn update(&mut self, ctx: engine::UpdateContext) {
		unsafe {
			let (r,g,b,_) = Color::rgb8(199, 145, 70).to_tuple();

			gl::disable(gl::Capability::DepthTest);
			gl::clear_color(r, g, b, 1.0);
			gl::clear(gl::COLOR_BUFFER_BIT);
		}

		self.camera.update(ctx.viewport);

		if ctx.input.tap() {
			let world_pos = self.camera.screen_to_world(ctx.input.position().extend(0.0));
			self.worms.push(Worm::new(world_pos.to_xy()));
		}

		self.shader.bind();
		self.shader.set_uniform("proj_view", self.camera.projection_view());

		self.mesh.clear();

		let screen_radius = WORLD_SCALE.hypot(WORLD_SCALE * self.camera.aspect());

		for worm in self.worms.iter_mut() {
			worm.update();

			let head_dist = worm.head().length();
			let tail_dist = worm.tail().length();

			let least_dist = tail_dist.min(head_dist);

			if least_dist > screen_radius {
				let centre_dir = -worm.head().normalize();
				worm.heading = centre_dir.to_angle();
				worm.heading_tendency = 0.0;
			}
		}

		for worm in self.worms.iter() {
			draw_worm(&mut self.mesh, worm);
		}

		self.mesh.draw(gl::DrawMode::Triangles);
	}

	fn drag_threshold(&self) -> Option<u32> { Some(30) }
}



#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vert {
	pub pos_part_a: Vec3,
	pub pos_b: Vec2,

	pub color: Vec3,
	pub side: f32,
	pub body_pos: f32,
	pub width: f32,
}

impl vertex::Vertex for Vert {
	fn descriptor() -> vertex::Descriptor {
		vertex::Descriptor::from(&[3, 2, 3, 1, 1, 1])
	}
}


fn subdivide_joints(joints: Vec<(Vec2, f32)>, ratio: f32) -> Vec<(Vec2, f32)> {
	let mut subdivided_joints: Vec<(Vec2, f32)> = Vec::with_capacity(joints.len() * 2 + 2);
	subdivided_joints.push(*joints.first().unwrap());

	for window in joints.windows(2) {
		if let &[(pos_a, part_a), (pos_b, part_b)] = window {
			let new_pos_0 = ratio.ease_linear(pos_a, pos_b);
			let new_pos_1 = ratio.ease_linear(pos_b, pos_a);
			let new_part_0 = ratio.ease_linear(part_a, part_b);
			let new_part_1 = ratio.ease_linear(part_b, part_a);

			subdivided_joints.extend_from_slice(&[
				(new_pos_0, new_part_0),
				(new_pos_1, new_part_1),
			]);
		}
	}

	subdivided_joints.push(*joints.last().unwrap());
	subdivided_joints
}


fn draw_worm(mesh: &mut DynamicMesh<Vert>, worm: &Worm) {
	let color = Color::rgb8(243, 159, 245).into();
	let cap_size = 0.3;
	let width = 0.2;

	let mut joints: Vec<(Vec2, f32)> = Vec::new();


	// Construct head
	let head_dir = Vec2::from_angle(worm.heading);
	let head0 = head_dir * cap_size + worm.joints[0];
	joints.push((head0, 0.0));

	// Construct body
	for &joint in worm.joints.iter() {
		joints.push((joint, 1.0));
	}

	// Construct tail
	let seg_count = worm.joints.len();
	let tailn1 = worm.joints[seg_count-2];
	let tail0 = worm.joints[seg_count-1];
	let tailv = (tail0 - tailn1).normalize();

	let tail2 = tail0 + tailv * cap_size;

	joints.extend_from_slice(&[
		(tail2, 2.0),
		(tail2 + tailv * 0.01, 2.0), // a dummy vert just for adjacency
	]);

	let subdivided_joints = joints;
	let subdivided_joints = subdivide_joints(subdivided_joints, 0.25);
	let subdivided_joints = subdivide_joints(subdivided_joints, 0.25);
	// let subdivided_joints = subdivide_joints(subdivided_joints, 0.25);

	// Draw
	let mut verts = Vec::new();

	let mut body_pos = 0.0;
	for window in subdivided_joints.windows(2) {
		if let &[(pos_a, part_a), (pos_b, _)] = window {
			verts.push(Vert {
				pos_part_a: pos_a.extend(part_a),
				pos_b,
				color,
				side: 1.0,
				body_pos,
				width,
			});

			verts.push(Vert {
				pos_part_a: pos_a.extend(part_a),
				pos_b,
				color,
				side: -1.0,
				body_pos,
				width,
			});

			body_pos += (pos_a - pos_b).length();
		}
	}

	mesh.add_tri_strip(&verts);

	// Draw eye + blink
	if worm.blink_timeout < 0.1 {
		return
	}

	let eye_pos = worm.joints[0] + head_dir.perp() * 0.05;
	let eye_width = 0.03;
	let eye_color = Vec3::zero();
	let eye_dir = (head_dir + head_dir.perp()).normalize();

	let eye_a = eye_pos + eye_dir * eye_width;
	let eye_b = eye_pos - eye_dir * eye_width;

	mesh.add_quad(&[
		Vert {
			pos_part_a: eye_a.extend(1.0),
			pos_b: eye_b,
			color: eye_color,
			side: 1.0,
			body_pos: 0.0,
			width: eye_width
		},

		Vert {
			pos_part_a: eye_a.extend(1.0),
			pos_b: eye_b,
			color: eye_color,
			side: -1.0,
			body_pos: 0.0,
			width: eye_width
		},

		Vert {
			pos_part_a: eye_b.extend(1.0),
			pos_b: eye_a,
			color: eye_color,
			side: 1.0,
			body_pos: 0.0,
			width: eye_width
		},

		Vert {
			pos_part_a: eye_b.extend(1.0),
			pos_b: eye_a,
			color: eye_color,
			side: -1.0,
			body_pos: 0.0,
			width: eye_width
		},
	]);
}



struct Worm {
	joints: [Vec2; 15],
	heading: f32,
	heading_tendency: f32,
	age: f32,
	blink_timeout: f32,
}

impl Worm {
	fn new(p: Vec2) -> Worm {
		let heading = rand() * 2.0 * PI;
		let mut joints = [Vec2::zero(); 15];

		let fact = rand();
		let coil_amt = 0.2 + fact * 1.0;
		let dist_offset = 0.4 - fact * 0.3;

		let reverse_coil = rand() > 0.5;
		let coil_dir = if reverse_coil { 1.0 } else { -1.0 };
		let coil_offset = (PI/2.0 + PI/64.0) * coil_dir;

		let joint_count = joints.len();
		let joint_count_f = joint_count as f32;

		for (i, joint) in joints.iter_mut().enumerate() {
			let ph = i as f32 / joint_count_f * 2.0 * PI * coil_amt * coil_dir;
			let dist = (1.0 - i as f32 / joint_count_f) * 0.6 + dist_offset;
			*joint = p + Vec2::from_angle(ph + heading + coil_offset) * dist;
		}

		Worm {
			joints,
			heading,
			heading_tendency: 0.0,
			age: 0.0,
			blink_timeout: rand() * 8.0 + 1.0
		}
	}

	fn head(&self) -> Vec2 {
		*self.joints.first().unwrap()
	} 

	fn tail(&self) -> Vec2 {
		*self.joints.last().unwrap()
	} 

	fn update(&mut self) {
		self.age += DT;
		self.blink_timeout -= DT;

		if self.blink_timeout < 0.0 {
			self.blink_timeout = rand() * 8.0 + 1.0;
		}

		let old_joints = self.joints;
		let head = old_joints[0];

		let heading_wiggle = (self.age * PI*4.0).sin() * PI/14.0;

		self.heading_tendency += (rand() * 2.0 - 1.0) * 2.0 * DT;
		self.heading_tendency *= (1.0 - 0.1 * DT);

		let tendency = (self.heading_tendency * 6.0 + heading_wiggle).clamp(-PI/6.0, PI/6.0);
		self.heading += tendency * DT;

		let target = head + Vec2::from_angle(self.heading) * 0.2;

		let joint_targets = iter::once(target)
			.chain(old_joints.iter().cloned());

		for (idx, (new, mut target)) in self.joints.iter_mut().zip(joint_targets).enumerate() {
			let v = target - *new;
			let dist = v.length();

			if dist > 0.01 {
				let perp = v.perp() / dist;
				let pos = idx as f32;

				let wiggle_amt = (pos/2.0-1.0).max(1.0).sqrt() / 4.0;
				let wiggle_offset = ((pos / 4.0 - self.age * 1.5) * PI).sin() * wiggle_amt;

				target += perp * wiggle_offset * dist;
			}

			*new = (2.0 * DT).ease_linear(*new, target);
		}
	}
}