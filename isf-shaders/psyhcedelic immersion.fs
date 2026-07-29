/*
{
    "CATEGORIES": [
        "Psychedelic",
        "Tunnel",
        "Optimized",
        "Texture"
    ],
    "DESCRIPTION": "A psychedelic tunnel shader, modified to include triplanar texture mapping, camera controls, and an enhanced post-processing pipeline based on the previous shader. The core chaotic animation and volumetric accumulation are preserved.",
    "ISFVSN": "2.0",
    "INPUTS": [
        { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0, "LABEL": "Animation Speed" },
        { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0, "LABEL": "Global Zoom" },
        { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5, "LABEL": "Transform Mode" },
        { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6, "LABEL": "Geometry Type" },
        { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0, "LABEL": "Chaos Strength" },
        { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0, "LABEL": "Chaos Speed" },
        { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 19, "MIN": 0, "MAX": 19, "LABEL": "Palette Mode" },
        { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0, "MAX": 3.0, "LABEL": "Global Brightness" },
        { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 3.0, "LABEL": "Global Contrast" },
        { "NAME": "Glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0, "LABEL": "Global Glow" },
        { "NAME": "Symmetry", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 4.0, "LABEL": "Symmetry Factor" },
        { "NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0, "LABEL": "Chaos Color Mix" },
        { "NAME": "Sharpness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0, "LABEL": "Sharpness" },
        { "NAME": "FalloffCurve", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.1, "MAX": 3.0, "LABEL": "Falloff Curve" },
        { "NAME": "CameraOrbit", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "LABEL": "Camera Orbit" },
        { "NAME": "CameraPitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.57, "MAX": 1.57, "LABEL": "Camera Pitch" },
        { "NAME": "CameraRoll", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "LABEL": "Camera Roll" },
        { "NAME": "FocusNear", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0, "LABEL": "Camera Focus Near" },
        { "NAME": "FocusFar", "TYPE": "float", "DEFAULT": 2.6, "MIN": 0.1, "MAX": 10.0, "LABEL": "Camera Focus Far" },
        { "NAME": "FOV", "TYPE": "float", "DEFAULT": 1.6, "MIN": 0.2, "MAX": 3.0, "LABEL": "Field of View" },
        { "NAME": "StepCount", "TYPE": "float", "DEFAULT": 60.0, "MIN": 1.0, "MAX": 128.0, "LABEL": "Raymarch Steps" },
        { "NAME": "Texture", "TYPE": "image" },
        { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0, "LABEL": "Texture Warp Intensity" },
        { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0, "LABEL": "Texture Scale" }
    ]
}
*/

#define BAILOUT 16.0
#define PI 3.14159265359
#define TAU 6.28318530718
#define R(a) mat2(cos(a+vec4(0,33,11,0)))

// Camera matrix for controlling viewpoint (from previous shader)
mat3 cameraMatrix(float orbit, float pitch, float roll) {
  float co = cos(orbit), so = sin(orbit);
  float cp = cos(pitch), sp = sin(pitch);
  float cr = cos(roll), sr = sin(roll);
  return mat3(
    co * cr + so * sp * sr, sr * cp, -so * cr + co * sp * sr,
    -co * sr + so * sp * cr, cr * cp, sr * so + co * sp * cr,
    so * cp, -sp, co * cp
  );
}

// A more robust palette function (from previous shader)
vec3 pal(float t_val, vec3 a, vec3 b, vec3 c, vec3 d) {
    return a + b * cos(TAU * (c * t_val + d));
}

// Function to get a color from one of the predefined palettes
vec3 getColorPalette(int mode, float t) {
   if (mode == 0) return pal(t, vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,1.0,1.0),vec3(0.0,0.33,0.67));
   if (mode == 1) return pal(t, vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,1.0,1.0),vec3(0.0,0.10,0.20));
   if (mode == 2) return pal(t, vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,0.0,1.0),vec3(0.0,0.33,0.67));
   if (mode == 3) return pal(t, vec3(0.8,0.5,0.4),vec3(0.2,0.4,0.2),vec3(2.0,1.0,1.0),vec3(0.0,0.25,0.45));
   if (mode == 4) return pal(t, vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,0.7,0.4),vec3(0.0,0.15,0.20));
   if (mode == 5) return pal(t, vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(2.0,1.0,0.0),vec3(0.5,0.20,0.25));
   if (mode == 6) return pal(t, vec3(0.8,0.8,0.5),vec3(0.5,0.5,0.5),vec3(1.0,1.0,1.0),vec3(0.0,0.15,0.20));
   if (mode == 7) return pal(t, vec3(1.0,0.5,0.5),vec3(0.5,0.5,0.5),vec3(0.5,0.8,1.0),vec3(0.3,0.5,0.8));
   if (mode == 8) return pal(t, vec3(1.0,0.5,0.5),vec3(0.2,0.4,0.4),vec3(2.0,1.0,1.0),vec3(0.0,0.1,0.2));
   if (mode == 9) return pal(t, vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,0.5,0.0),vec3(0.8,0.9,0.3));
   if (mode == 10) return pal(t, vec3(0.0,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,1.0,0.5),vec3(0.8,0.9,0.3));
   if (mode == 11) return pal(t, vec3(0.5,0.5,0.8),vec3(0.5,0.5,0.5),vec3(1.0,0.7,0.4),vec3(0.0,0.15,0.20));
   if (mode == 12) return pal(t, vec3(0.6,0.6,0.6),vec3(0.4,0.4,0.4),vec3(1.0,1.0,1.0),vec3(0.0,0.33,0.67));
   if (mode == 13) return pal(t, vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,0.7,0.4),vec3(0.0,0.15,0.20));
   if (mode == 14) return pal(t, vec3(0.8,0.8,0.5),vec3(0.5,0.5,0.5),vec3(1.0,1.0,1.0),vec3(0.0,0.15,0.20));
   if (mode == 15) return pal(t, vec3(1.0,0.5,0.5),vec3(0.5,0.5,0.5),vec3(0.5,0.8,1.0),vec3(0.3,0.5,0.8));
   if (mode == 16) return pal(t, vec3(1.0,0.5,0.5),vec3(0.2,0.4,0.4),vec3(2.0,1.0,1.0),vec3(0.0,0.1,0.2));
   if (mode == 17) return pal(t, vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,0.5,0.0),vec3(0.8,0.9,0.3));
   if (mode == 18) return pal(t, vec3(0.0,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,1.0,0.5),vec3(0.8,0.9,0.3));
   if (mode == 19) return pal(t, vec3(0.5,0.5,0.8),vec3(0.5,0.5,0.5),vec3(1.0,0.7,0.4),vec3(0.0,0.15,0.20));
   return vec3(0.0);
}

// Triplanar texture mapping from the previous shader
vec3 triplanarTexture(vec3 p, float scale) {
  vec3 blend = normalize(abs(p));
  blend = pow(blend, vec3(4.0));
  blend /= dot(blend, vec3(1.0));
  vec3 tex_coord = p * scale;

  vec3 tx = texture2D(Texture, tex_coord.zy).rgb;
  vec3 ty = texture2D(Texture, tex_coord.xz).rgb;
  vec3 tz = texture2D(Texture, tex_coord.xy).rgb;

  return tx * blend.x + ty * blend.y + tz * blend.z;
}

// --- SDF-like functions from the ShaderToy original ---
float shapeSpikeFractal(vec3 p) {
  float d = 0.0;
  for (int i = 0; i < int(StepCount); i++) {
    p = abs(p) / dot(p, p + 0.001) - 0.5;
    p *= 0.95;
    d += length(p);
  }
  return d / 20.0;
}

float shapeChaos(vec3 p, float chaos) {
  return (sin(p.x*3. + TIME*ChaosSpeed) + sin(p.y*4. + TIME*ChaosSpeed*1.2) + sin(p.z*5. + TIME*ChaosSpeed*0.8)) * chaos;
}

float scene(vec3 p, int geo, float chaos, float mixAmt) {
  float base;
  if (geo == 0) base = length(p) - 1.0;
  else if (geo == 1) {
    vec2 q = vec2(length(p.xz)-1.0, p.y);
    base = length(q) - 0.3;
  }
  else if (geo == 2) base = shapeSpikeFractal(p * 1.2);
  else base = shapeSpikeFractal(p);
  return mix(base, shapeChaos(p, chaos), mixAmt);
}

vec3 applyTransform(vec3 p, int mode, float chaos, float sym, float chspd) {
  p *= max(sym, 0.001);
  if (mode == 1) p = abs(p);
  else if (mode == 2) p += sin(p * 3.0 + TIME * chspd) * chaos * 0.3;
  else if (mode == 3) {
    p += sin(p * (1.0 + chaos * 2.0) + TIME * chspd) * chaos * 0.5;
    p = fract(p * 1.5) - 0.75;
  }
  if (mode == 4 || mode == 5) {
    float a = atan(p.z, p.x);
    float r = length(p.xz);
    float spin = TIME * chspd * (mode == 4 ? 0.2 : 0.3);
    a += spin;
    p.x = cos(a) * r;
    p.z = sin(a) * r;
  }
  return p;
}

void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
  uv *= FOV;
  float t = TIME * Speed;

  vec3 ro = vec3(0.0, 0.0, -3.0);
  vec3 rd = normalize(vec3(uv * Zoom, 1.0));
  // Apply camera controls (from previous shader)
  rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

  // Apply texture warping to the ray origin (from previous shader)
  vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
  vec3 roWarped = ro + warp * TextureWarp;

  vec3 col = vec3(0.0);
  float dist = 0.0;

  int mode = int(TransformMode);
  int geo = int(GeometryType);
  float chaos = ChaosIntensity;
  float chaosMix = ChaosMix;
  float sym = Symmetry;
  float chspd = ChaosSpeed;
  float br = Brightness;
  float ct = Contrast;
  float glow = Glow;
  int pal = int(ColorPaletteMode);
  float sharp = Sharpness;
  float falloff = FalloffCurve;

  // Raymarching loop
  for (int i = 0; i < int(StepCount); i++) {
    vec3 p = roWarped + dist * rd;
    p = applyTransform(p, mode, chaos, sym, chspd);
    float d = scene(p, geo, chaos, chaosMix);
    d = max(abs(d), 0.01);

    // Apply sharpness and focus factors (from previous shader)
    float fade = exp(-float(i)*0.03*sharp);
    float focus = smoothstep(FocusNear, FocusFar, dist);

    vec3 palCol = getColorPalette(pal, p.z + t * 0.1);
    vec3 texCol = triplanarTexture(p * TextureScale, 1.0);
    float b = 0.005 / (0.01 + d * falloff);

    // Accumulate color with texture and focus
    col += mix(palCol, texCol, ChaosMix) * b * fade * focus;

    dist += d;
    if (dist > BAILOUT) break;
  }

  // Post-processing (unified from both shaders)
  float pulse = sin(TIME * ChaosSpeed) * 0.5 + 0.5;
  col *= 1.0 + 0.3 * pulse;
  col = (col - 0.5) * ct + 0.5;
  col *= br;
  col += glow * (vec3(1.0) - col) * col;

  gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
