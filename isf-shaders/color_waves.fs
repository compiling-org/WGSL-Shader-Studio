/*{
  "CREDIT": "by VIDVOX",
  "CATEGORIES": [
    "Generator"
  ],
  "INPUTS": [
    {
      "NAME": "rate",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 10.0,
      "DEFAULT": 1.0
    },
    {
      "NAME": "intensity",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 1.0,
      "DEFAULT": 0.5
    }
  ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    float time = TIME * rate;
    
    float r = sin(uv.x * 10.0 + time) * intensity;
    float g = sin(uv.y * 10.0 + time * 1.1) * intensity;
    float b = sin((uv.x + uv.y) * 10.0 + time * 0.9) * intensity;
    
    gl_FragColor = vec4(r, g, b, 1.0);
}