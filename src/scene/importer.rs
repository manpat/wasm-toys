use crate::scene::types::*;
use crate::EngineResult;
use std::convert::TryInto;

use common::*;
use failure::ensure;

pub fn load_toy_file(data: &[u8]) -> EngineResult<ToyFile> {
	let reader = ToyReader { buf: data };
	reader.read_all()
}

struct ToyReader<'data> { buf: &'data [u8] }

impl<'d> ToyReader<'d> {
	fn read_all(mut self) -> EngineResult<ToyFile> {
		self.expect_tag(b"TOY\x01")?;

		let num_meshes = self.read_u16()? as usize;
		let mut meshes = Vec::with_capacity(num_meshes);
		for _ in 0..num_meshes {
			meshes.push(self.read_mesh()?);
		}

		let num_entities = self.read_u16()? as usize;
		let mut entities = Vec::with_capacity(num_entities);
		for _ in 0..num_entities {
			entities.push(self.read_entity()?);
		}

		let num_scenes = self.read_u16()? as usize;
		let mut scenes = Vec::with_capacity(num_scenes);
		for _ in 0..num_scenes {
			scenes.push(self.read_scene()?);
		}

		Ok(ToyFile {
			scenes,
			entities,
			meshes,
		})
	}

	fn read_mesh(&mut self) -> EngineResult<MeshData> {
		self.expect_tag(b"MESH")?;

		let num_vertices = self.read_u16()? as usize;
		let mut vertices = Vec::with_capacity(num_vertices);
		for _ in 0..num_vertices {
			vertices.push(self.read_vec3()?);
		}

		let wide_indices = num_vertices >= 256;

		let num_indices = self.read_u16()? as usize;
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
			ensure!(num_points == num_vertices, "Color layer '{}' different size to vertex list");
			
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
		self.expect_tag(b"ENTY")?;
		
		Ok(EntityData {
			name: self.read_string()?,
			position: self.read_vec3()?,
			rotation: self.read_quat()?,
			scale: self.read_vec3()?,
			mesh_id: self.read_u16()?,
		})
	}

	fn read_scene(&mut self) -> EngineResult<SceneData> {
		self.expect_tag(b"SCNE")?;
		let name = self.read_string()?;
		let num_entities = self.read_u16()? as usize;
		let mut entities = Vec::with_capacity(num_entities);
		for _ in 0..num_entities {
			entities.push(self.read_u16()?);
		}

		Ok(SceneData {
			name,
			entities
		})
	}

	fn expect_tag(&mut self, tag: &[u8; 4]) -> EngineResult<()> {
		ensure!(self.buf.len() >= 4, "Unexpected EOF while expecting tag {:?}", tag);
		ensure!(&self.buf[..4] == tag, "Expected tag {:?}", tag);
		self.buf = &self.buf[4..];
		Ok(())
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