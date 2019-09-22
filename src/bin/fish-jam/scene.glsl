precision highp float;

uniform mat4 proj_view;

attribute vec3 position;
attribute vec3 color;

varying vec3 v_position;
varying vec3 v_color;

void main() {
    gl_Position = proj_view * vec4(position, 1.0);
	gl_PointSize = 5.0;
    v_position = position;
    v_color = color;
}


/* @@@ */


precision highp float;

varying vec3 v_position;
varying vec3 v_color;

void main() {
    // gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
    gl_FragColor = vec4(v_color, 1.0);
}