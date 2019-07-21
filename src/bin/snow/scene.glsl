precision highp float;

attribute vec3 position;

uniform mat4 u_proj_view;

varying vec2 v_position;

void main() {
	gl_Position = u_proj_view * vec4(position, 1.0);
	v_position = position.xz;
}


/* @@@ */

precision highp float;

uniform vec4 u_color;
varying vec2 v_position;

float sample_distance() {
	return max(length(v_position) - 0.5, 0.0);
}

void main() {
	float alpha = 1.0 - sample_distance()/2.0;
	gl_FragColor = vec4(u_color.rgb*alpha, alpha);
}