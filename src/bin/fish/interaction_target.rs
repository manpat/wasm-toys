use engine::prelude::*;
use engine::scene;

use crate::player_controller::PlayerController;


const INTERACTION_DIST: f32 = 3.0;
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


pub fn interaction_targets_in_range(scene: scene::SceneRef, ply: &PlayerController) -> Vec<InteractionTarget> {
	let player_pos = ply.pos.to_xz();
	let player_fwd = ply.rot.forward().to_xz().normalize();

	let mut its = scene.entities()
		.filter(|e| e.name.starts_with("IT_"))
		.map(move |e| {
			let diff = e.position.to_xz() - player_pos;
			let dist = diff.length();
			let angle = (diff.dot(player_fwd) / dist).acos();
			let it = InteractionTarget {
				name: e.name.clone(),
				pos: e.position,
				suitability: None,
			};

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
