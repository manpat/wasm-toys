use crate::graphics::TextureID;
use crate::engine::{EngineResult, Ticks};
use failure::{bail, format_err};

use common::math::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SpriteID(usize);

pub const NULL_SPRITE: SpriteID = SpriteID(0);

impl SpriteID {
	pub fn is_null(self) -> bool { self.0 == 0 }
}

pub fn hack_sprite_id(i: usize) -> SpriteID { SpriteID(i) }

#[derive(Copy, Clone, Debug)]
pub struct AnimationFrame {
	pub texture: TextureID,
	pub pos: Vec2i,
	pub size: Vec2i,
}

pub enum AnimationData {
	Static(AnimationFrame),
	Looping(Vec<AnimationFrame>),
	PingPong(Vec<AnimationFrame>),
}



pub struct Sprite {
	pub animation_data: AnimationData,
	pub frame_duration: Ticks,
}

impl Sprite {
	pub fn sample(&self, time: Ticks) -> AnimationFrame {
		use self::AnimationData::*;
		
		assert!(self.frame_duration > 0);

		match self.animation_data {
			Static(frame) => frame,
			Looping(ref frames) => {
				let idx = (time / self.frame_duration) as usize;
				frames[idx % frames.len()]
			}

			PingPong(ref _frames) => unimplemented!()
		}
	}

	pub fn get_frame(&self, frame: usize) -> Option<AnimationFrame> {
		use self::AnimationData::*;
		
		match self.animation_data {
			Static(frame) => Some(frame),
			Looping(ref frames) => frames.get(frame).cloned(),
			PingPong(ref frames) => frames.get(frame).cloned(),
		}
	}

	pub fn num_frames(&self) -> usize {
		use self::AnimationData::*;

		match self.animation_data {
			Static(_) => 1,
			Looping(ref frames) => frames.len(),
			PingPong(ref frames) => frames.len(),
		}
	}
}



pub struct SpriteRegistry {
	sprites: Vec<Sprite>,
}

impl SpriteRegistry {
	pub fn new() -> Self {
		SpriteRegistry {
			sprites: Vec::new(),
		}
	}

	pub fn register_static_sprite(&mut self, texture: TextureID, pos: Vec2i, size: Vec2i) -> SpriteID {
		let animation_data = AnimationData::Static(AnimationFrame { texture, pos, size });
		self.sprites.push(Sprite { animation_data, frame_duration: 1 });

		console_log!("Sprite registered {} ({:?}), {:?} {:?}", self.sprites.len(), texture, pos, size);
		SpriteID(self.sprites.len())
	}

	pub fn register_looped_sprite(&mut self, frame_duration: Ticks, frames: Vec<AnimationFrame>) -> SpriteID {
		console_log!("Looped sprite registered {} {:?}", self.sprites.len(), frames);

		let animation_data = AnimationData::Looping(frames);
		self.sprites.push(Sprite { animation_data, frame_duration });
		SpriteID(self.sprites.len())
	}

	pub fn get_sprite(&self, id: SpriteID) -> EngineResult<&Sprite> {
		if id.is_null() {
			bail!("Trying to get null sprite");
		}

		self.sprites.get(id.0 - 1)
			.ok_or_else(|| format_err!("Trying to get invalid sprite {:?}", id))
	}

	pub fn get_sprite_mut(&mut self, id: SpriteID) -> EngineResult<&mut Sprite> {
		if id.is_null() {
			bail!("Trying to get null sprite");
		}

		self.sprites.get_mut(id.0 - 1)
			.ok_or_else(|| format_err!("Trying to get invalid sprite {:?}", id))
	}

	pub fn get_frame_at_time(&self, id: SpriteID, time: Ticks) -> EngineResult<AnimationFrame> {
		let spr = self.get_sprite(id)?;
		Ok(spr.sample(time))
	}
}