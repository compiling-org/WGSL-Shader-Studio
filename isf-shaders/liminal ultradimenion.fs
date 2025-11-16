/*{
  "CATEGORIES": ["Psychedelic", "Fractal", "Volumetric", "Visuals"],
  "DESCRIPTION": "Merged & fixed ISF shader. TextureMode: 0=Off, 1=Normal blend, 2=Texture-dominant. Texture drives color + displacement when enabled. ColorPulse/Shift/Palette sliders actually affect final color.",
  "ISFVSN": "2.0",
  "INPUTS": [
    { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0 },
    { "NAME": "zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0 },
    { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5 },
    { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6 },
    { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0 },
    { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 1, "MIN": 0, "MAX": 19 },
    { "NAME": "colorShift", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "colorPulse", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 5.0 },
    { "NAME": "morphing", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
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
    { "NAME": "TextureMode", "TYPE": "float", "DEFAULT": 1, "MIN": 0, "MAX": 2 },
    { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0 },
    { "NAME": "shimmer", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "camX", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0 },
    { "NAME": "camY", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0 },
    { "NAME": "camZ", "TYPE": "float", "DEFAULT": 0.0, "MIN": -10.0, "MAX": 10.0 },
    { "NAME": "shake", "TYPE": "float", "DEFAULT": 0.05, "MIN": 0.0, "MAX": 0.2 }
  ]
}*/

#define MAX_STEPS 48
#define BAILOUT 16.0
#define PI 3.14159265
#define TAU 6.2831853

mat3 cameraMatrix(float orbit, float pitch, float roll) {
  float co = cos(orbit), so = sin(orbit);
  float cp = cos(pitch), sp = sin(pitch);
  float cr = cos(roll), sr = sin(roll);
  return mat3(
    co*cr + so*sp*sr,  sr*cp, -so*cr + co*sp*sr,
    -co*sr + so*sp*cr, cr*cp,  sr*so + co*sp*cr,
    so*cp,            -sp,     co*cp
  );
}

vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
  return a + b * cos(TAU * (c * t + d));
}

vec3 getColorPalette(int mode, float t) {
  // Several simple palettes; mode picks one (keeps range safe)
  int m = int(clamp(float(mode), 0.0, 7.0)); // <-- FIXED: clamp as float then cast to int
  if (m == 0) return pal(t, vec3(0.5), vec3(0.5), vec3(1.0), vec3(0.0));
  if (m == 1) return pal(t, vec3(0.5,0.25,0.6), vec3(0.5), vec3(1.0,1.2,0.8), vec3(0.0));
  if (m == 2) return pal(t, vec3(0.2,0.6,0.5), vec3(0.6), vec3(1.0,0.8,1.3), vec3(0.1));
  if (m == 3) return pal(t, vec3(0.6,0.2,0.3), vec3(0.4), vec3(1.0,1.0,0.7), vec3(0.2));
  if (m == 4) return pal(t, vec3(0.5,0.7,0.2), vec3(0.5), vec3(1.0,0.9,1.1), vec3(0.3));
  if (m == 5) return pal(t, vec3(0.7,0.4,0.6), vec3(0.45), vec3(1.0,1.1,0.6), vec3(0.05));
  if (m == 6) return pal(t, vec3(0.4,0.6,0.7), vec3(0.5), vec3(1.0,0.7,1.2), vec3(0.0));
  return pal(t, vec3(0.5), vec3(0.5), vec3(1.0), vec3(0.0));
}

vec3 triplanarTexture(vec3 p, float scale) {
  // create stable [0,1] coords from 3D point using fract; uses Texture uniform
  vec3 blend = normalize(abs(p) + 0.0001);
  blend = pow(blend, vec3(4.0));
  blend /= (blend.x + blend.y + blend.z);
  vec2 xz = fract(p.zy * scale);
  vec2 yz = fract(p.xz * scale);
  vec2 xy = fract(p.xy * scale);
  vec3 tx = texture2D(Texture, xz).rgb;
  vec3 ty = texture2D(Texture, yz).rgb;
  vec3 tz = texture2D(Texture, xy).rgb;
  return tx * blend.x + ty * blend.y + tz * blend.z;
}

float shapeSpikeFractal(vec3 p, float steps) {
  float d = 0.0;
  int maxI = int(clamp(steps, 1.0, 128.0));
  for (int i = 0; i < 128; i++) {
    if (i >= maxI) break;
    p = abs(p) / dot(p, p + 0.001) - 0.5;
    p *= 0.95;
    d += length(p);
  }
  return d * 0.05;
}

float shapeChaos(vec3 p, float chaos, float chspd) {
  return (sin(p.x*3.0 + TIME*chspd) + sin(p.y*4.0 + TIME*chspd*1.2) + sin(p.z*5.0 + TIME*chspd*0.8)) * chaos;
}

float sceneSigned(vec3 p, int geo, float chaos, float mixAmt, float steps) {
  float base;
  if (geo == 0) base = length(p) - 1.0;
  else if (geo == 1) {
    vec2 q = vec2(length(p.xz)-1.0, p.y);
    base = length(q) - 0.3;
  } else if (geo == 2) base = shapeSpikeFractal(p * 1.2, steps);
  else base = shapeSpikeFractal(p, steps);
  return mix(base, shapeChaos(p, chaos, ChaosSpeed), clamp(mixAmt, 0.0, 1.0));
}

vec3 applyTransform(vec3 p, int mode, float chaos, float sym, float chspd, float morph, float shake) {
  p *= max(sym, 0.0001);
  // small procedural jitter influenced by morph & chaos
  p += sin(p.yzx * 3.0 + TIME * chspd) * chaos * 0.02 * morph;
  p += (fract(p * 1.2345) - 0.5) * shake * 0.1;
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

int safeInt(float v) { return int(max(0.0, floor(v + 0.5))); }

void main() {
  // safe parameter extraction
  int tMode = safeInt(TextureMode); // 0 off, 1 normal, 2 dominant
  int transMode = safeInt(TransformMode);
  int geoMode = safeInt(GeometryType);
  int palMode = safeInt(ColorPaletteMode);
  float steps = max(1.0, StepCount);

  // texture-driven time warp (sample a 1D sweep across texture to let image affect animation)
  vec2 texTimeUV = vec2(fract(TIME * 0.02), 0.5);
  float texTimeSample = texture2D(Texture, texTimeUV).r;
  float t = TIME * Speed * (1.0 + 0.6 * texTimeSample * float(tMode > 0));

  // screen coordinates + camera offsets
  vec2 uv = (gl_FragCoord.xy + vec2(camX, camY) - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
  float dynamicZoom = 1.0 + 0.25 * sin(t * 0.05);
  vec2 pScreen = uv / (zoom * dynamicZoom);
  pScreen += pScreen * sin(dot(pScreen, pScreen) * 20.0 - t) * 0.04 * morphing;

  // camera
  vec3 ro = vec3(camX * 0.12, camY * 0.12, -3.0 + camZ);
  vec3 rd = normalize(vec3(pScreen * FOV, 1.0));
  rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

  // camera warp from texture when enabled
  vec3 camTex = triplanarTexture(ro * TextureScale + vec3(t * 0.01), 1.0) - 0.5;
  ro += camTex * TextureWarp * (tMode == 2 ? 1.5 : (tMode == 1 ? 0.6 : 0.0));

  // accumulation
  vec3 col = vec3(0.0);
  float dist = 0.0;

  int loopMax = int(min(float(MAX_STEPS), max(1.0, steps)));
  for (int i = 0; i < MAX_STEPS; i++) {
    if (i >= loopMax) break;

    vec3 p = ro + rd * dist;

    // texture displacement inside scene if enabled
    if (tMode > 0) {
      vec3 warp = triplanarTexture(p * TextureScale * 0.6 + vec3(t * 0.02), 1.0) - 0.5;
      p += warp * TextureWarp * (tMode == 2 ? 1.2 : 0.5);
    }

    // apply transforms
    p = applyTransform(p, transMode, ChaosIntensity, Symmetry, ChaosSpeed, morphing, shake);

    // distance estimate
    float d = sceneSigned(p, geoMode, ChaosIntensity, ChaosMix, steps);
    d = max(abs(d), 0.01);

    // shading factors
    float fade = exp(-float(i) * 0.03 * Sharpness);
    float focus = smoothstep(FocusFar, FocusNear, dist); // near/far control
    focus = clamp(focus, 0.0, 1.0);

    // palette driven by position/time
    vec3 palCol = getColorPalette(palMode, p.z * 0.12 + t * 0.08 + float(i) * 0.02);

    // texture color sample
    vec3 texCol = triplanarTexture(p * TextureScale + vec3(t * 0.02), 1.0);

    // mix logic: when texture dominant, let texture set base color; otherwise combine
    vec3 baseCol;
    if (tMode == 0) {
      baseCol = palCol;
    } else if (tMode == 1) {
      // normal blend: bias towards texture more strongly than before
      baseCol = mix(palCol, texCol, 0.65);
    } else {
      // texture-dominant: texture drives hue & luminance, palette tints it
      baseCol = texCol * (0.6 + 0.8 * palCol);
    }

    // further modulate palette by texture luminance so the image input strongly affects final look
    float texL = dot(texCol, vec3(0.2126, 0.7152, 0.0722));
    baseCol *= 0.6 + 0.8 * texL;

    // add shimmer & pulse
    vec3 shimmerCol = shimmer * vec3(
      sin((p.x + p.y) * 20.0 + t),
      sin((p.y + p.z) * 18.0 + t * 1.2),
      sin((p.z + p.x) * 16.0 + t * 0.9)
    ) * 0.5;

    float b = 0.005 / (0.01 + d * FalloffCurve);

    // accumulate
    float pulse = 1.0 + colorPulse * sin(t * 1.2 + float(i) * 0.05);
    col += (baseCol + shimmerCol) * b * fade * focus * pulse;

    dist += d;
    if (dist > BAILOUT) break;
  }

  // final post-processing: ensure colorPulse, colorShift, palette, saturation actually visible
  // global pulse (subtle)
  col *= (1.0 + 0.25 * colorPulse * (0.5 + 0.5 * sin(TIME * ChaosSpeed)));

  // contrast
  col = (col - 0.5) * Contrast + 0.5;

  // brightness & glow
  col *= Brightness * (1.0 + Glow * 0.25);

  // saturation
  float lum = dot(col, vec3(0.2126, 0.7152, 0.0722));
  col = mix(vec3(lum), col, saturation);

  // color shift (swaps towards bgr based on slider)
  col = mix(col, col.bgr, clamp(colorShift, 0.0, 1.0));

  // clamp and output
  col = clamp(col, 0.0, 1.0);
  gl_FragColor = vec4(col, 1.0);
}
