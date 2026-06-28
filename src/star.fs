// This shader is based from https://www.shadertoy.com/view/Ml2XDt
// Adaptations and modifications _were_ made to conform visual preference
#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
uniform float uTime;
out vec4 finalColor;
void main( ) {
    vec2 p = fragTexCoord - vec2(0.5);

    float b = ceil(atan(p.x, p.y) * 2400.0);
    float h = cos(b);
    float z = h / dot(p,p);

    float f = exp(fract(z + h * b + uTime) * -3.0) / z;
    finalColor = vec4(vec3(f), 1.0);
}