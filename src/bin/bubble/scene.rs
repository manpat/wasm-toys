use engine::graphics::DynamicMesh;
use engine::graphics::vertex::ColorVertex as Vertex;
use common::*;

pub type Mesh = DynamicMesh<Vertex>;

pub fn init() -> (Mesh, Mesh) {
	let scene = parse_ply(include_str!("scene.ply"));
	let portal = parse_ply(include_str!("portal.ply"));

	(scene, portal)
}

fn parse_ply(src: &str) -> Mesh {
	let vert_count: usize = src.lines()
		.find(|l| l.starts_with("element vertex"))
		.and_then(|l| l.split_whitespace().nth(2))
		.and_then(|n| n.parse().ok())
		.unwrap();

	let face_count: usize = src.lines()
		.find(|l| l.starts_with("element face"))
		.and_then(|l| l.split_whitespace().nth(2))
		.and_then(|n| n.parse().ok())
		.unwrap();

	let body_start = src.lines()
		.skip_while(|l| !l.starts_with("end_header"))
		.skip(1);

	let vert_iter = body_start.clone().take(vert_count);
	let face_iter = body_start
		.skip(vert_count)
		.take(face_count);

	let verts = vert_iter
		.map(|l| {
			let mut it = l.split_whitespace();
			let pos = Vec3::new(
				it.next().and_then(|n| n.parse().ok()).unwrap(),
				it.next().and_then(|n| n.parse().ok()).unwrap(),
				it.next().and_then(|n| n.parse().ok()).unwrap(),
			);

			let color = Color::rgb8(
				it.next().and_then(|n| n.parse().ok()).unwrap_or(0),
				it.next().and_then(|n| n.parse().ok()).unwrap_or(0),
				it.next().and_then(|n| n.parse().ok()).unwrap_or(0),
			);

			Vertex::new(pos, color.into())
		})
		.collect::<Vec<_>>();

	let mut indices: Vec<u16> = face_iter.clone()
		.filter(|l| l.starts_with("3"))
		.flat_map(|l| {
			let mut it = l.split_whitespace().skip(1).filter_map(|n| n.parse().ok());
			let results = [
				it.next().unwrap(),
				it.next().unwrap(),
				it.next().unwrap(),
			];

			(0..3).map(move |i| results[i])
		})
		.collect();

	let quad_indices = face_iter
		.filter(|l| l.starts_with("4"))
		.flat_map(|l| {
			let mut it = l.split_whitespace()
				.skip(1)
				.filter_map(|n| n.parse::<u16>().ok());

			let results = [
				it.next().unwrap(),
				it.next().unwrap(),
				it.next().unwrap(),
				it.next().unwrap(),
			];

			let indices = [0, 1, 2, 0, 2, 3];

			(0..6).map(move |i| results[indices[i]])
		});

	indices.extend(quad_indices);

	let mut mesh = Mesh::new();
	mesh.add_geometry(&verts, &indices);
	mesh
}