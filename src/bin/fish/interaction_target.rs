use engine::prelude::*;
use engine::scene;

use crate::scene_view::*;
use crate::player_controller::PlayerController;


const INTERACTION_DIST: f32 = 2.5;
const INTERACTION_ARC: f32 = PI / 8.0;

const VISIBILITY_DIST: f32 = 4.0;


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Suitability {
	Nearby,
	Interactible
}


#[derive(Debug)]
pub struct InteractionTarget {
	pub name: String,
	pub pos: Vec3,

	pub suitability: Option<Suitability>,
}


pub fn interaction_targets_in_scene<'s>(file: &'s scene::ToyFile, scene_name: &str) -> impl Iterator<Item=InteractionTarget> + 's {
	entities_in_scene(file, scene_name)
		.filter(|e| e.name.starts_with("IT_"))
		.map(|e| InteractionTarget {
			name: e.name.clone(),
			pos: e.position,
			suitability: None,
		})
}


pub fn interaction_targets_in_range(file: &scene::ToyFile, scene_name: &str, ply: &PlayerController) -> Vec<InteractionTarget> {
	let player_pos = ply.pos.to_xz();
	let player_fwd = ply.rot.forward().to_xz();

	let mut its = interaction_targets_in_scene(file, scene_name)
		.map(move |mut it| {
			let diff = it.pos.to_xz() - player_pos;
			let dist = diff.length();
			let angle = (diff.dot(player_fwd) / dist).acos();
			(angle, dist, it)
		})
		.filter(|(_, dist, _)| *dist < VISIBILITY_DIST)
		.collect::<Vec<_>>();

	its.sort_by_key(|(_, dist, _)| ordify(dist));

	its.into_iter()
		.map(|(angle, dist, mut it)| {
			if angle > INTERACTION_ARC || dist > INTERACTION_DIST {
				it.suitability = Some(Suitability::Nearby);
			} else {
				it.suitability = Some(Suitability::Interactible);
			}

			it
		})
		.collect()
}
