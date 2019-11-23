precision highp float;

uniform mat4 proj_view;
uniform float particle_scale;

attribute vec3 position;
attribute vec4 part_info;

varying vec4 v_part_info;

void main() {
    gl_Position = proj_view * vec4(position, 1.0);

    float vel = length(part_info.rgb) * 2.0;
	gl_PointSize = particle_scale / (1.0 + vel*vel);

	v_part_info = part_info;
}


/* @@@ */

precision highp float;

varying vec4 v_part_info;

void main() {
	float dist = 1.0 - 2.0 * length(gl_PointCoord - vec2(0.5));
	if (dist < 0.0) {
		discard;
	}

    gl_FragColor = vec4((v_part_info.rgb)*0.5 + 0.5, 1.0);
}