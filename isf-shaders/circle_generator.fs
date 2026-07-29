/*{
  "CREDIT": "Circle Generator",
  "CATEGORIES": [
    "Generator"
  ],
  "INPUTS": [
    {
      "NAME": "radius",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 0.5,
      "DEFAULT": 0.2
    },
    {
      "NAME": "center_x",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 1.0,
      "DEFAULT": 0.5
    },
    {
      "NAME": "center_y",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 1.0,
      "DEFAULT": 0.5
    },
    {
      "NAME": "color_r",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 1.0,
      "DEFAULT": 1.0
    },
    {
      "NAME": "color_g",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 1.0,
      "DEFAULT": 1.0
    },
    {
      "NAME": "color_b",
      "TYPE": "float",
      "MIN": 0.0,
      "MAX": 1.0,
      "DEFAULT": 1.0
    }
  ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    vec2 center = vec2(center_x, center_y);
    
    float dist = distance(uv, center);
    float circle = 1.0 - smoothstep(radius - 0.01, radius + 0.01, dist);
    
    gl_FragColor = vec4(color_r * circle, color_g * circle, color_b * circle, circle);
}