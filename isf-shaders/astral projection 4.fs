/*{
  "CATEGORIES": ["Fractal", "Psychedelic", "Volumetric", "3D"],
  "DESCRIPTION": "Combined fractal shader: base fractal & transforms from shader 1, with animation, pulse, glitch, palette and camera from shader 2, full parameter merge.",
  "ISFVSN": "2.0",
  "INPUTS": [
    { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0 },
    { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 5.0 },
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

    { "NAME": "RaymarchSteps", "TYPE": "float", "DEFAULT": 99.0, "MIN": 20.0, "MAX": 200.0 },

    { "NAME": "Texture", "TYPE": "image" },
    { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0 },

    { "NAME": "Morph", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "PulseRate", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0 },
    { "NAME": "PulseStrength", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },

    { "NAME": "Saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "FractalIterations", "TYPE": "float", "DEFAULT": 5.0, "MIN": 1.0, "MAX": 10.0 },

    { "NAME": "ShakeAmount", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "Glitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },

    { "NAME": "CameraAngle", "TYPE": "point2D", "DEFAULT": [0.5, 0.5] },
    { "NAME": "CameraPos", "TYPE": "point2D", "DEFAULT": [0.0, 0.0] }
  ]
}*/

#define MAX_STEPS 200
#define BAILOUT 16.0
#define PI 3.14159265359

// Camera matrix from first shader
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

// Palette from first shader
vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
  return a + b * cos(6.2831 * (c * t + d));
}

// Extended color palette blending from first shader, with mode
vec3 getColorPalette(int mode, float t) {
  return pal(t,
    vec3(0.5 + 0.4*sin(float(mode)*0.5), 0.6 + 0.3*cos(float(mode)*1.2), 0.4 + 0.5*sin(float(mode)*0.9)),
    vec3(0.4),
    vec3(1.0,1.3,0.7),
    vec3(0.1,0.2,0.3)
  );
}

// Triplanar texture sampling from first shader
vec3 triplanarTexture(vec3 p, float scale) {
  vec3 blend = normalize(abs(p));
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

// Shape: spike fractal from first shader
float shapeSpikeFractal(vec3 p) {
  float d = 0.0;
  for (int i = 0; i < 128; i++) {
    if (i >= int(FractalIterations)) break;
    p = abs(p) / dot(p, p + 0.001) - 0.5;
    p *= 0.95;
    d += length(p);
  }
  return d / 20.0;
}

// Chaos shape addition from first shader
float shapeChaos(vec3 p, float chaos) {
  return (sin(p.x*3. + TIME*ChaosSpeed) + sin(p.y*4. + TIME*ChaosSpeed*1.2) + sin(p.z*5. + TIME*ChaosSpeed*0.8)) * chaos;
}

// Scene distance function mixing geometry from first shader and chaos
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

// Apply transformations from first shader with all modes supported
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

// Palette from second shader
vec3 palette(float t, float type) {
    if (type < 0.5) return vec3(sin(t*6.0), cos(t*3.0), sin(t*2.0)) * 0.5 + 0.5;
    if (type < 1.5) return vec3(fract(t*2.3), fract(t*5.7), fract(t*9.1));
    if (type < 2.5) return vec3(abs(sin(t*3.1)), abs(sin(t*2.1+1.0)), abs(sin(t*4.1+2.0)));
    if (type < 3.5) return vec3(cos(t*10.0), cos(t*5.0), sin(t*7.0)) * 0.5 + 0.5;
    if (type < 4.5) return vec3(sin(t*5.0), sin(t*7.0+1.0), sin(t*9.0+2.0)) * 0.5 + 0.5;
    if (type < 5.5) return vec3(sin(t*8.0 + sin(t*3.0)), cos(t*4.0), sin(t*6.0));
    return vec3(t, t*t, t*t*t)*0.25 + 0.25;
}

// Adjust color with brightness, saturation, contrast from second shader
vec3 adjust(vec3 color, float br, float sat, float con) {
    color = mix(vec3(0.5), color, con);
    color = mix(vec3(dot(color, vec3(0.299, 0.587, 0.114))), color, sat);
    return color * br;
}

// Glitch noise from second shader
float glitchNoise(vec2 uv) {
    uv *= vec2(40.0, 4.0);
    return fract(sin(dot(uv, vec2(12.9898,78.233))) * 43758.5453);
}

// Rotate vector p around axis a by angle r
vec3 R(vec3 p, vec3 a, float r) {
    return mix(dot(p,a)*a,p,cos(r)) + sin(r)*cross(p,a);
}

void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
  uv *= FOV;

  float t = TIME * Speed;

  // Camera position and orientation (first shader + second shader shake & morph)
  vec3 ro = vec3(0.0, 0.0, -3.0);
  vec3 rd = normalize(vec3(uv * Zoom, 1.0));
  rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

  // Add triplanar texture warp to camera position for psychedelic surface displacement
  vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
  vec3 roWarped = ro + warp * TextureWarp;

  // Additional shake from second shader applied to cameraPos
  float shakeX = sin(t * 13.0) * ShakeAmount;
  float shakeY = cos(t * 17.0) * ShakeAmount;

  // Calculate offset camera position from second shader
  float yaw = (CameraAngle.x - 0.5) * 6.283;
  float pitch = (CameraAngle.y - 0.5) * 3.1415;
  vec3 camTarget = vec3(0.0);
  vec3 camOffset = vec3(
      sin(yaw) * cos(pitch),
      sin(pitch),
      cos(yaw) * cos(pitch)
  ) * (4.0 - Morph * 3.0);
  vec3 camPos = camOffset + vec3(CameraPos.x * 5.0 + shakeX, CameraPos.y * 5.0 + shakeY, 0.0);

  // Final ray direction with second shader camera adjustments
  vec3 fwd = normalize(camTarget - camPos);
  vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), fwd));
  vec3 up = cross(fwd, right);
  vec3 d = normalize(fwd + uv.x * right + uv.y * up);

  // Initialize color accumulator and distance
  vec3 color = vec3(0.0);
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

  int fractalIters = int(clamp(FractalIterations, 1.0, 10.0));
  int maxSteps = int(clamp(RaymarchSteps, 20.0, float(MAX_STEPS)));

  for (int i = 0; i < MAX_STEPS; i++) {
    if (i >= maxSteps) break;

    vec3 p = roWarped + dist * rd;

    // Apply base fractal transform and morph from second shader on fractal point
    if (float(GeometryType) < 3.0) {
      p = applyTransform(p, mode, chaos, sym, chspd);
    }

    // Base fractal shape + chaos mix from first shader
    float dVal = scene(p, geo, chaos, chaosMix);

    // Add fractal iteration detail (from second shader)
    float s = 2.0;
    float e = 0.0;
    vec3 q = p;
    for (int j = 0; j < 10; j++) {
      if (j >= fractalIters) break;
      e = 2.0 / min(dot(q,q), 1.0);
      q = abs(q)*e - vec3(3.0, 20.0, 9.0 + Morph * 4.0);
      s *= e;
    }

    dVal += abs(length(q.xy - clamp(q.xy, -0.5, 0.5)) / s) + 0.005;

    dVal = max(abs(dVal), 0.01);

    // Fade and focus
    float fade = exp(-float(i)*0.03*sharp);
    float focus = smoothstep(FocusNear, FocusFar, dist);

    // Color palette and texture blending
    vec3 palCol = getColorPalette(pal, p.z + t * 0.1);
    vec3 texCol = triplanarTexture(p * TextureScale, 1.0);

    // Pulse effect from second shader
    float pulse = sin(TIME * ChaosSpeed) * 0.5 + 0.5;
    float pulse2 = sin(float(i) * PulseRate + t) * PulseStrength;

    // Mix palettes with pulse and chaosMix
    vec3 mixedPal = mix(palCol, palette(log(s)*0.5 + pulse2, ColorPaletteMode), 0.5);

    // Final color accumulation with pulse and fade
    color += mix(mixedPal, texCol, chaosMix) * (0.005 / (0.01 + dVal * falloff)) * fade * focus * (1.0 + 0.3 * pulse);

    dist += dVal;
    if (dist > BAILOUT) break;
  }

  // Final color correction
  color = adjust(pow(color, vec3(4.0)), br * glow, Saturation, ct);

  // Glitch effect from second shader
  if (Glitch > 0.0) {
    float n = glitchNoise(gl_FragCoord.xy) * Glitch;
    color += n * vec3(1.0, 0.2, 0.0);
  }

  gl_FragColor = vec4(clamp(color, 0.0, 1.0), 1.0);
}
