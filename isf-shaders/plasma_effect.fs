/*{
  "CREDIT": "Plasma Effect",
  "CATEGORIES": [
    "Generator"
  ],
  "INPUTS": [
    {
      "NAME": "speed",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 5.0,
      "DEFAULT": 1.0
    },
    {
      "NAME": "scale",
      "TYPE": "float",
      "MIN": 0.1,
      "MAX": 10.0,
      "DEFAULT": 3.0
    },
    {
      "NAME": "intensity",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 2.0,
      "DEFAULT": 1.0
    }
  ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    float time = TIME * speed;
    
    float v = 0.0;
    v += sin((uv.x + time) * scale);
    v += sin((uv.y + time * 0.8) * scale * 0.8);
    v += sin((uv.x + uv.y + time * 0.5) * scale * 0.6);
    v += sin(sqrt(uv.x * uv.x + uv.y * uv.y) * scale + time * 0.3);
    
    v *= intensity;
    
    vec3 col;
    col.r = sin(v * 3.14159) * 0.5 + 0.5;
    col.g = sin(v * 3.14159 * 1.2) * 0.5 + 0.5;
    col.b = sin(v * 3.14159 * 1.5) * 0.5 + 0.5;
    
    gl_FragColor = vec4(col, 1.0);
}