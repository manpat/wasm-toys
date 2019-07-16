precision highp float;

uniform mat4 u_proj_view;

uniform sampler2D u_voxel_data;
uniform float u_voxel_chunk_size;

attribute vec3 position;
attribute vec3 normal;
attribute vec3 voxel_pos;

varying vec3 v_position;
varying vec3 v_normal;
varying float v_voxel_id;

float voxel_data(vec3 pos) {
	bool oob = any(lessThan(pos, vec3(0.0))) || any(greaterThanEqual(pos, vec3(u_voxel_chunk_size)));
	if (oob) {
		return 0.0;
	}

	vec2 uv = pos.zy / u_voxel_chunk_size;
	uv.x += pos.x / (u_voxel_chunk_size*u_voxel_chunk_size);
	return texture2D(u_voxel_data, uv).x;
}

void main() {
	float voxel = voxel_data(voxel_pos);

	// discard empty voxels
	if (voxel == 0.0) {
		gl_Position = vec4(0.0, 0.0, 100.0, 1.0);
		return;
	}

	// discard obscured faces
	if (voxel_data(voxel_pos + normal) != 0.0) {
		gl_Position = vec4(0.0, 0.0, 100.0, 1.0);
		return;
	}

    gl_Position = u_proj_view * vec4(position, 1.0);

    v_position = position;
    v_normal = normal;
    v_voxel_id = voxel * 255.0;
}


/* @@@ */

precision highp float;

varying vec3 v_position;
varying vec3 v_normal;
varying float v_voxel_id;

void main() {
	float dither_size = 0.25;

	bvec3 dither = greaterThan(mod(v_position + dither_size/4.0, dither_size), vec3(dither_size/2.0));
	if (any(dither)) {
		discard;
	}


	float bit_0 = floor(mod(v_voxel_id, 2.0));
	float bit_1 = floor(mod((v_voxel_id - bit_0)/2.0, 2.0));
	float bit_2 = floor(mod((v_voxel_id - bit_0 - bit_1)/4.0, 2.0));

    gl_FragColor = vec4(
    	bit_0,
    	bit_1,
    	bit_2,
		1.0
	);
}