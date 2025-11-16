/*{
  "CATEGORIES": ["Fractal", "Psychedelic", "Volumetric", "Optimized"],
  "DESCRIPTION": "Merged: psychedelic Shadertoy fractal driven as the core distance generator, enriched by the volumetric ISF engine: triplanar texturing, chaos, symmetry, geometry modes, camera transforms, glow, sharpness and falloff.",
  "ISFVSN": "2.0",
  "INPUTS": [
    { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0 },
    { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0 },
    { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5 },
    { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6 },
    { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0 },
    { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 19, "MIN": 0, "MAX": 19 },
    { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0, "MAX": 3.0 },
    { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 3.0 },
    { "NAME": "Glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "Symmetry", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "Sharpness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0 },
    { "NAME": "FalloffCurve", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.1, "MAX": 3.0 },
    { "NAME": "CameraOrbit", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14 },
    { "NAME": "CameraPitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.57, "MAX": 1.57 },
    { "NAME": "CameraRoll", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14 },
    { "NAME": "FocusNear", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0 },
    { "NAME": "FocusFar", "TYPE": "float", "DEFAULT": 2.6, "MIN": 0.1, "MAX": 10.0 },
    { "NAME": "FOV", "TYPE": "float", "DEFAULT": 1.6, "MIN": 0.2, "MAX": 3.0 },
    { "NAME": "StepCount", "TYPE": "float", "DEFAULT": 6, "MIN": 1, "MAX": 60 },
    { "NAME": "Texture", "TYPE": "image" },
    { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0 },

    
    { "NAME": "psy_Speed", "TYPE": "float", "DEFAULT": 0.01, "MIN": 0.01, "MAX": 2.0 },
    { "NAME": "psy_Zoom", "TYPE": "float", "DEFAULT": 1.30, "MIN": 0.1, "MAX": 5.0 },
    { "NAME": "psy_Morphing", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "psy_Geometry", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.5, "MAX": 3.0 },
    { "NAME": "psy_ColorPulse", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 5.0 },
    { "NAME": "psy_ColorPalette", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 3.0, "LABELS": ["Rainbow","Fire","Acid","Neon"] }
  ]
} */

#define MAX_STEPS 48
#define BAILOUT 16.0
#define PI 3.14159265359

// -- small helpers
mat3 cameraMatrix(float orbit, float pitch, float roll) {
  float co = cos(orbit), so = sin(orbit);
  float cp = cos(pitch), sp = sin(pitch);
  float cr = cos(roll), sr = sin(roll);
  return mat3(
    co * cr + so * sp * sr,  sr * cp, -so * cr + co * sp * sr,
   -co * sr + so * sp * cr,  cr * cp,  sr * so + co * sp * cr,
    so * cp,                -sp,       co * cp
  );
}

vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
  return a + b * cos(6.28318 * (c * t + d));
}

vec3 getColorPalette(int mode, float t) {
  // combined rich palette bank (supports many modes)
  t = fract(t);
  if (mode < 0) mode = 0;
  // simple bank -- reuse pal-style base plus some manuals
  if (mode < 4) {
    // small set: rainbow / acid / neon / fire-ish
    if (mode == 0) return pal(t, vec3(0.5), vec3(0.5), vec3(1.0), vec3(0.0));
    if (mode == 1) return vec3(min(1.0,t*1.5), min(1.0, max(0.0,t-0.3)), 0.0);
    if (mode == 2) return fract(vec3(t*0.61, t*0.83, t*0.47));
    return abs(fract(vec3(t*3.0, t*5.0, t*7.0))*2.0 - 1.0);
  }
  // procedural variations for higher palette indices (mimic ISF input 0..19)
  float shift = float(mode) * 0.123;
  return pal(t + shift, vec3(0.5 + 0.4*sin(float(mode)*0.5), 0.6 + 0.3*cos(float(mode)*1.2), 0.4 + 0.5*sin(float(mode)*0.9)),
                    vec3(0.4),
                    vec3(1.0,1.3,0.7),
                    vec3(0.1,0.2,0.3));
}

vec3 triplanarTexture(vec3 p, float scale) {
  // cheap triplanar blend using fract sampling of Texture
  vec3 blend = normalize(abs(p) + 1e-6);
  blend = pow(blend, vec3(4.0));
  blend /= dot(blend, vec3(1.0));

  vec2 xz = fract(p.zy * scale);
  vec2 yz = fract(p.xz * scale);
  vec2 xy = fract(p.xy * scale);

  vec3 tx = texture2D(Texture, xz).rgb;
  vec3 ty = texture2D(Texture, yz).rgb;
  vec3 tz = texture2D(Texture, xy).rgb;

  return tx * blend.x + ty * blend.y + tz * blend.z;
}

// --- volumetric / geometry primitives & fractal engines

float shapeSpikeFractal(vec3 p, int iters, float globalScale) {
  float d = 0.0;
  // iterate with early-exit possibility
  for (int i = 0; i < 128; i++) {
    if (i >= iters) break;
    // stable denom, cheap sphere inversion / formula inspired by original
    float denom = dot(p, p) + 0.001;
    p = abs(p) / denom - 0.5;
    p *= 0.95 * globalScale;
    d += length(p);
    if (d > 1e6) break;
  }
  return d / 20.0;
}

float shapeChaos(vec3 p, float chaos) {
  // psychedelic moving noise-like modifier
  return (sin(p.x*3. + TIME*ChaosSpeed) + sin(p.y*4. + TIME*ChaosSpeed*1.2) + sin(p.z*5. + TIME*ChaosSpeed*0.8)) * chaos;
}

// --- Psychedelic 2D fractal (from Shadertoy) adapted to drive volumetric DF
// This is the "driving core" the user requested: it remains the main pattern generator
float psychedelicBase(vec3 p) {
  // Use XY slice as main fractal plane but fold in Z to modulate
  vec2 uv = p.xy;
  // camera-like movement & driving speed from psy_ inputs
  uv += vec2(sin(TIME * psy_Speed * 0.5), cos(TIME * psy_Speed * 0.3)) * 0.3;
  uv *= psy_Zoom;

  // morphing rotation
  float angle = TIME * psy_Morphing * 0.2;
  float s = sin(angle), c = cos(angle);
  mat2 rot = mat2(c, -s, s, c);
  uv = rot * uv;

  // iterate fractal 2D field (small fixed iterations for speed)
  float minDist = 100.0;
  vec2 p2 = uv;
  int iters = int(min(12.0, StepCount)); // cap iterations for performance
  for (int i = 0; i < 12; i++) {
    if (i >= iters) break;
    // logistic-like fold used in original shadertoy
    float denom = max(0.2, dot(p2, p2));
    p2 = abs(p2) / denom - psy_Geometry;
    // subtle rotate over time
    float a = TIME * 0.1;
    float sa = sin(a), ca = cos(a);
    p2 = mat2(ca, -sa, sa, ca) * p2;
    float ld = length(p2);
    minDist = min(minDist, ld);
  }

  // now fold in z as modulation for volumetric depth
  // z influences color/time mapping and tiny offset to minDist
  minDist += abs(p.z) * 0.15;

  return minDist; // lower = closer geometry in DF sense
}

// scene DF: combine psychedelicBase as main driver with geometry modes and chaos
float sceneDF(vec3 p, int geo, float chaosIntensity, float chaosMix, int psyPaletteMode) {
  float base = 0.0;

  // primary: psychedelic base drives the volumetric "surface" (so it stays central)
  base = psychedelicBase(p);

  // geometry variants (from original ISF shader)
  if (geo == 0) {
    // sphere-ish fallback
    base = length(p) - 1.0 + base * 0.2;
  } else if (geo == 1) {
    vec2 q = vec2(length(p.xz) - 1.0, p.y);
    base = length(q) - 0.3 + base * 0.25;
  } else if (geo == 2) {
    // spike fractal variant using more iterations
    base = shapeSpikeFractal(p * 1.2, int(StepCount * 2.0), 1.0) * 0.8 + base * 0.6;
  } else if (geo == 3) {
    base = shapeSpikeFractal(p, int(StepCount * 1.5), 1.0) * 0.9 + base * 0.5;
  } else {
    // combine spike + psychedelic as complex hybrid
    float s = shapeSpikeFractal(p * 0.9, int(StepCount * 1.2), 1.0);
    base = mix(base, s, 0.6);
  }

  // additive chaotic displacement (small)
  float chaos = shapeChaos(p, chaosIntensity) * chaosMix;
  base = mix(base, base + chaos * 0.5, chaosMix);

  // Keep DF positive-ish and avoid zero:
  return base;
}

// spatial transforms applied before evaluating sceneDF (symmetry, folding, spin, etc.)
vec3 applyTransform(vec3 p, int mode, float chaos, float sym, float chspd) {
  // symmetry scale
  p *= max(sym, 0.001);

  if (mode == 1) {
    p = abs(p); // mirror-symmetry
  } else if (mode == 2) {
    p += sin(p * 3.0 + TIME * chspd) * chaos * 0.3;
  } else if (mode == 3) {
    p += sin(p * (1.0 + chaos * 2.0) + TIME * chspd) * chaos * 0.5;
    p = fract(p * 1.5) - 0.75;
  } else if (mode == 4 || mode == 5) {
    float a = atan(p.z, p.x);
    float r = length(p.xz);
    float spin = TIME * chspd * (mode == 4 ? 0.2 : 0.3);
    a += spin;
    p.x = cos(a) * r;
    p.z = sin(a) * r;
  }

  return p;
}

// cheap tone mapping/gamma tweak
vec3 tone(vec3 c, float br, float ct, float glow) {
  c = (c - 0.5) * ct + 0.5;
  c *= br * (1.0 + glow);
  return c;
}

void main() {
  // normalized pixel coords
  vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
  uv *= FOV;

  // time scaled for volumetric movement (user Speed controls global animation)
  float t = TIME * Speed;

  // camera origin & ray direction
  vec3 ro = vec3(0.0, 0.0, -3.0);
  vec3 rd = normalize(vec3(uv * Zoom, 1.0));
  rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

  // apply small triplanar warp to camera origin (texture-driven)
  vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
  vec3 roWarped = ro + warp * TextureWarp;

  // parameters pulled from inputs
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
  int steps = int(min(MAX_STEPS, max(1.0, StepCount)));

  // raymarch and accumulate volumetric scattering-like color
  vec3 col = vec3(0.0);
  float dist = 0.0;

  // precompute a pulsing factor (psychedelic influence)
  float pulse = sin(TIME * psy_ColorPulse) * 0.5 + 0.5;

  for (int i = 0; i < MAX_STEPS; i++) {
    if (i >= steps) break;

    vec3 p = roWarped + dist * rd;

    // apply global transforms (symmetry, spin, warps)
    p = applyTransform(p, mode, chaos, sym, chspd);

    // evaluate the scene DF where psychedelicBase drives the geometry
    float d = sceneDF(p, geo, chaos, chaosMix, int(psy_ColorPalette));

    // ensure safe non-zero
    d = max(abs(d), 0.0001);

    // progressive fade per step (sharpness controls how quickly contributions fall)
    float fade = exp(-float(i) * 0.03 * sharp);

    // depth focus falloff
    float focus = smoothstep(FocusNear, FocusFar, dist);

    // color from palette driven by p.z and TIME with psychedelic pulse
    vec3 palCol = getColorPalette(pal, p.z + t * 0.1 + pulse * 0.25);

    // texture color from triplanar mapping (adds visible texture influence)
    vec3 texCol = triplanarTexture(p * TextureScale, 1.0);

    // mix palette and texture but let psychedelic driving slightly bias mixing by pulse
    float texMix = 0.4 + 0.6 * pulse; // more psychedelic => more palette
    vec3 mixedCol = mix(texCol, palCol, texMix);

    // brightness contribution inversely proportional to DF (closer = brighter)
    // cheaper approximation than pow
    float b = 0.005 / (0.01 + d * falloff);

    // accumulate
    col += mixedCol * b * fade * focus;

    dist += d;

    if (dist > BAILOUT) break;
  }

  // post-processing color tweaks: pulse-driven vibrance + contrast/brightness/glow
  col *= 1.0 + 0.3 * pulse * psy_ColorPulse * 0.2;
  col = tone(col, br, ct, glow);

  // final vignette / falloff
  float v = 1.0 - 0.5 * dot((gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y, (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y);
  v = pow(clamp(v, 0.0, 1.0), FalloffCurve);
  col *= v;

  // clamp and output
  gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
