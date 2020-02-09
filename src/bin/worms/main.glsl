precision highp float;

uniform mat4 proj_view;

attribute vec3 pos_part_a;
attribute vec2 pos_b;
attribute vec3 color;
attribute float side;
attribute float body_pos;
attribute float width;

varying vec3 v_color;
varying float v_body_pos;
varying float v_edge_pos;
varying float v_part;

void main() {
	vec2 pos_a = pos_part_a.xy;

	vec2 dir = normalize(pos_b - pos_a);
	vec2 normal = vec2(dir.y, -dir.x) * side * width;

	gl_Position = proj_view * vec4(pos_a + normal, 0.0, 1.0);
	v_color = color;
	v_body_pos = body_pos;
	v_edge_pos = side;
	v_part = pos_part_a.z;

	// if (v_part < 1.0) {
	// 	v_color = vec3(1.0, 0.0, 0.0);
	// }
}


/* @@@ */


precision highp float;

varying vec3 v_color;
varying float v_body_pos;
varying float v_edge_pos;
varying float v_part;

void main() {
	float pi_2 = 3.1415926 / 2.0;

	float edge_dist = abs(v_edge_pos);

	float curve = cos(edge_dist * pi_2);
	float pos = fract(v_body_pos - 0.4 - curve * 0.1);

	if (pos > 0.5) {
		gl_FragColor = vec4(v_color, 1.0);
	} else {
		gl_FragColor = vec4(v_color * 0.95, 1.0);
	}

	// Head and tail
	float cap_dist = length(vec2(edge_dist, 1.0 - v_part));
	if (cap_dist > 1.0) {
		discard;
	}
}