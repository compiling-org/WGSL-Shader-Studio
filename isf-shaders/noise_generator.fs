/*{
  "CREDIT": "Noise Generator",
  "CATEGORIES": [
    "Generator"
  ],
  "INPUTS": [
    {
      "NAME": "scale",
      "TYPE": "float",
      "MIN": 0.1,
      "MAX": 10.0,
      "DEFAULT": 2.0
    },
    {
      "NAME": "speed",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 5.0,
      "DEFAULT": 1.0
    }
  ]
}*/

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898,78.233))) * 43758.5453123);
}

void main() {
    vec2 uv = isf_FragNormCoord;
    float time = TIME * speed;
    
    vec2 noisy_uv = uv * scale + time * 0.1;
    float noise = random(noisy_uv);
    
    gl_FragColor = vec4(noise, noise, noise, 1.0);
}