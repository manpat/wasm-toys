precision highp float;

attribute vec3 position;
attribute vec3 color;

uniform mat4 u_proj_view;

varying vec3 v_color;

void main() {
	gl_Position = u_proj_view * vec4(position, 1.0);
	gl_PointSize = 3.0;
	v_color = color;
}


/* @@@ */

precision highp float;

varying vec3 v_color;

// float sample_distance() {
// 	return max(length(v_position) - 0.5, 0.0);
// }

void main() {
	float alpha = 1.0;
	gl_FragColor = vec4(v_color.rgb*alpha, alpha);
}