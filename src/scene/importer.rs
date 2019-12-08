use crate::scene::types::*;
use crate::EngineResult;
use std::convert::TryInto;

use common::*;
use failure::{ensure, bail, format_err};

const SCENE_VERSION: u8 = 2;

pub fn load_toy_file(data: &[u8]) -> EngineResult<ToyFile> {
	let reader = ToyReader { buf: data };
	reader.read_all()
}


type Tag = [u8; 4];

struct ToyReader<'data> { buf: &'data [u8] }

impl<'d> ToyReader<'d> {
	fn read_all(mut self) -> EngineResult<ToyFile> {
		self.read_magic()?;

		let mut meshes = Vec::new();
		let mut entities = Vec::new();
		let mut scenes = Vec::new();

		while !self.buf.is_empty() {
			let (tag, mut section) = self.read_section()?;

			let to_err = |e| format_err!("While parsing '{}' section: {}", tag_to_string(&tag), e);

			match &tag {
				b"SCNE" => scenes.push(section.read_scene().map_err(to_err)?),
				b"MESH" => meshes.push(section.read_mesh().map_err(to_err)?),
				b"ENTY" => entities.push(section.read_entity().map_err(to_err)?),
				_ => bail!("Unexpected tag '{}' encountered", tag_to_string(&tag))
			}
		}

		Ok(ToyFile {
			scenes,
			entities,
			meshes,
		})
	}

	fn read_magic(&mut self) -> EngineResult<()> {
		ensure!(&self.buf[..3] == b"TOY", "Expected magic string");
		self.buf = &self.buf[3..];

		let version = self.read_u8()?;
		ensure!(version == SCENE_VERSION, "Version mismatch ({}/{})", version, SCENE_VERSION);

		Ok(())
	}

	fn read_section(&mut self) -> EngineResult<(Tag, ToyReader)> {
		let tag = self.read_tag()?;
		let section_size = self.read_u32()? as usize;
		ensure!(section_size <= self.buf.len(), "Invalid section size for '{}'", tag_to_string(&tag));

		let (section, rest) = self.buf.split_at(section_size);
		self.buf = rest;

		Ok((tag, ToyReader{ buf: section }))
	}

	fn read_mesh(&mut self) -> EngineResult<MeshData> {
		let num_vertices = self.read_u16()? as usize;
		let mut vertices = Vec::with_capacity(num_vertices);
		for _ in 0..num_vertices {
			vertices.push(self.read_vec3()?);
		}

		let wide_indices = num_vertices >= 256;

		let num_triangles = self.read_u16()? as usize;
		let num_indices = num_triangles * 3;
		let indices;

		if wide_indices {
			let mut indices_buf = Vec::with_capacity(num_indices);
			for _ in 0..num_indices {
				indices_buf.push(self.read_u16()?);
			}
			indices = MeshIndices::U16(indices_buf);

		} else {
			let mut indices_buf = Vec::with_capacity(num_indices);
			for _ in 0..num_indices {
				indices_buf.push(self.read_u8()?);
			}
			indices = MeshIndices::U8(indices_buf);
		}

		let num_color_layers = self.read_u8()? as usize;
		let mut color_data = Vec::with_capacity(num_color_layers);
		for _ in 0..num_color_layers {
			self.expect_tag(b"MDTA")?;

			let layer_name = self.read_string()?;
			let num_points = self.read_u16()? as usize;
			ensure!(num_points == num_vertices, "Color layer '{}' different size to vertex list", layer_name);

			let mut layer_data = Vec::with_capacity(num_points);
			for _ in 0..num_points {
				layer_data.push(self.read_vec4()?);
			}

			color_data.push(MeshColorData {
				name: layer_name,
				data: layer_data,
			})
		}

		Ok(MeshData {
			positions: vertices,
			indices,
			color_data
		})
	}

	fn read_entity(&mut self) -> EngineResult<EntityData> {
		Ok(EntityData {
			name: self.read_string()?,
			position: self.read_vec3()?,
			rotation: self.read_quat()?,
			scale: self.read_vec3()?,
			mesh_id: self.read_u16()?,
		})
	}

	fn read_scene(&mut self) -> EngineResult<SceneData> {
		let name = self.read_string()?;
		let num_entities = self.read_u32()? as usize;
		let mut entities = Vec::with_capacity(num_entities);
		for _ in 0..num_entities {
			entities.push(self.read_u32()?);
		}

		Ok(SceneData {
			name,
			entities
		})
	}

	fn expect_tag(&mut self, tag: &Tag) -> EngineResult<()> {
		ensure!(self.buf.len() >= 4, "Unexpected EOF while expecting tag '{}'", tag_to_string(tag));
		ensure!(&self.buf[..4] == tag, "Expected tag '{}'", tag_to_string(tag));
		self.buf = &self.buf[4..];
		Ok(())
	}

	fn read_tag(&mut self) -> EngineResult<Tag> {
		ensure!(self.buf.len() >= 4, "Unexpected EOF while expecting tag");
		let (tag, rest) = self.buf.split_at(4);
		self.buf = rest;
		Ok(tag.try_into()?)
	}

	fn read_u8(&mut self) -> EngineResult<u8> {
		ensure!(self.buf.len() >= 1, "Unexpected EOF while expecting u8");
		let b = self.buf[0];
		self.buf = &self.buf[1..];
		Ok(b)
	}

	fn read_u16(&mut self) -> EngineResult<u16> {
		ensure!(self.buf.len() >= 2, "Unexpected EOF while expecting u16");
		let (b, rest) = self.buf.split_at(2);
		self.buf = rest;
		Ok(u16::from_le_bytes(b.try_into()?))
	}

	fn read_u32(&mut self) -> EngineResult<u32> {
		ensure!(self.buf.len() >= 4, "Unexpected EOF while expecting u32");
		let (b, rest) = self.buf.split_at(4);
		self.buf = rest;
		Ok(u32::from_le_bytes(b.try_into()?))
	}

	fn read_f32(&mut self) -> EngineResult<f32> {
		Ok(f32::from_bits(self.read_u32()?))
	}

	fn read_vec3(&mut self) -> EngineResult<Vec3> {
		Ok(Vec3::new(
			self.read_f32()?,
			self.read_f32()?,
			self.read_f32()?
		))
	}

	fn read_vec4(&mut self) -> EngineResult<Vec4> {
		Ok(Vec4::new(
			self.read_f32()?,
			self.read_f32()?,
			self.read_f32()?,
			self.read_f32()?
		))
	}

	fn read_quat(&mut self) -> EngineResult<Quat> {
		Ok(Quat::from_raw(
			self.read_f32()?,
			self.read_f32()?,
			self.read_f32()?,
			self.read_f32()?
		))
	}

	fn read_string(&mut self) -> EngineResult<String> {
		let length = self.read_u8()? as usize;

		ensure!(self.buf.len() >= length, "Unexpected EOF while reading string");
		let (utf8, tail) = self.buf.split_at(length);
		self.buf = tail;

		std::str::from_utf8(utf8)
			.map(Into::into)
			.map_err(Into::into)
	}
}


fn tag_to_string(tag: &Tag) -> String {
	unsafe {
		std::str::from_utf8_unchecked(tag).into()
	}
}