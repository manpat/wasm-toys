precision highp float;

attribute vec3 position;
attribute vec2 uv;
attribute float lifetime;

uniform mat4 u_proj_view;
uniform float u_particle_scale;

varying vec3 v_color;

void main() {
    gl_Position = u_proj_view * vec4(position, 1.0);

    mat4 scale_mat = u_proj_view;

    // Remove translation
    scale_mat[0].w = 0.0;
    scale_mat[1].w = 0.0;

    vec4 point_scale = scale_mat * vec4(u_particle_scale, u_particle_scale, position.z, 1.0);
    point_scale /= point_scale.w;

    gl_PointSize = max(max(point_scale.x, point_scale.y), 2.0);
    v_color = vec3(1.0);
}


/* @@@ */

precision highp float;

varying vec3 v_color;

void main() {
    gl_FragColor = vec4(v_color.x, gl_PointCoord, 1.0);
}