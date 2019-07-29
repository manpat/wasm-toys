precision highp float;

attribute vec3 position;
attribute float sprite_stage;

uniform mat4 u_proj_view;
uniform float u_particle_scale;

varying vec3 v_position;
varying float v_sprite_stage;

void main() {
	gl_Position = u_proj_view * vec4(position, 1.0);
	v_position = position;

	mat4 scale_mat = u_proj_view;

	// Remove translation
	scale_mat[0].w = 0.0;
	scale_mat[1].w = 0.0;

	// Try to figure out particle size in pixels after projection
	vec4 point_scale = scale_mat * vec4(u_particle_scale, u_particle_scale, position.z, 1.0);
	point_scale /= point_scale.w;
	gl_PointSize = max(point_scale.x, point_scale.y);

	v_sprite_stage = floor(sprite_stage);
}


/* @@@ */

precision highp float;

varying vec3 v_position;
varying float v_sprite_stage;

float sample_distance() {
	return max(length(v_position.xz) - 0.5, 0.0);
}

float almost_eq(vec2 a, vec2 b) {
	return float(length(a - b) < 0.1);
}

float sample_snowflake(vec2 uv) {
	vec2 cell = floor(uv * 3.0);

	// Spawn
	if (v_sprite_stage < 0.0) {
		return almost_eq(cell, vec2(1.0, 1.0));
	}

	// Falling
	if (v_sprite_stage < 3.0) {
		return mod(cell.x + cell.y + v_sprite_stage - 1.0, 2.0);
	}

	// Resting
	return almost_eq(cell, vec2(1.0, 2.0));
}

void main() {
	float margin = 0.04;
	vec2 uv = mix(vec2(margin), vec2(1.0 - margin), gl_PointCoord);

	float alpha = (1.0 - sample_distance()/1.5)
		* (1.0 - clamp(v_position.y - 1.0, 0.0, 1.0));

	alpha = clamp(alpha, 0.0, 1.0);

	float sample = sample_snowflake(uv) * alpha;
	if (sample == 0.0) {
		discard;
	} 

	gl_FragColor = vec4(sample);
}