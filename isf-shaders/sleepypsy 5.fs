/*{
  "CATEGORIES": ["Fractal", "Psychedelic", "Volumetric", "3D"],
  "DESCRIPTION": "wrapped with fractal transforms and textures for complexity.",
  "ISFVSN": "2.0",
  "INPUTS": [
    { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0 },
    { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 5.0 },

    { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5 },
    { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6 },
    { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.1, "MAX": 4.0 },
    { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 19 },

    { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0, "MAX": 3.0 },
    { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 3.0 },
    { "NAME": "Glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "Symmetry", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0, "MAX": 1.0 },

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
    { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 10.0 },

    { "NAME": "Morph", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "PulseRate", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0 },
    { "NAME": "PulseStrength", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },

    { "NAME": "Saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "FractalIterations", "TYPE": "float", "DEFAULT": 5.0, "MIN": 1.0, "MAX": 10.0 },
    { "NAME": "FractalDominance", "TYPE": "float", "DEFAULT": 0.7, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "TextureDominance", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },

    { "NAME": "ShakeAmount", "TYPE": "float", "DEFAULT": 0.18, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "Glitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

#define MAX_STEPS 200.0  // Change MAX_STEPS to a float
#define BAILOUT 16.0
#define PI 3.14159265359

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

vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
  return a + b * cos(6.2831 * (c * t + d));
}

vec3 getColorPalette(int mode, float t) {
  return pal(t,
    vec3(0.5 + 0.4*sin(float(mode)*0.5), 0.6 + 0.3*cos(float(mode)*1.2), 0.4 + 0.5*sin(float(mode)*0.9)),
    vec3(0.4),
    vec3(1.0,1.3,0.7),
    vec3(0.1,0.2,0.3)
  );
}

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

vec3 palette(float t, float type) {
  if (type < 0.5) return vec3(sin(t*6.0), cos(t*3.0), sin(t*2.0)) * 0.5 + 0.5;
  if (type < 1.5) return vec3(fract(t*2.3), fract(t*5.7), fract(t*9.1));
  if (type < 2.5) return vec3(abs(sin(t*3.1)), abs(sin(t*2.1+1.0)), abs(sin(t*4.1+2.0)));
  if (type < 3.5) return vec3(cos(t*10.0), cos(t*5.0), sin(t*7.0)) * 0.5 + 0.5;
  if (type < 4.5) return vec3(sin(t*5.0), sin(t*7.0+1.0), sin(t*9.0+2.0)) * 0.5 + 0.5;
  if (type < 5.5) return vec3(sin(t*8.0 + sin(t*3.0)), cos(t*4.0), sin(t*6.0));
  return vec3(t, t*t, t*t*t)*0.25 + 0.25;
}

vec3 adjust(vec3 color, float br, float sat, float con) {
  color = mix(vec3(0.5), color, con);
  color = mix(vec3(dot(color, vec3(0.299, 0.587, 0.114))), color, sat);
  return color * br;
}

float glitchNoise(vec2 uv) {
  uv *= vec2(40.0, 4.0);
  return fract(sin(dot(uv, vec2(12.9898,78.233))) * 43758.5453);
}

vec3 R(vec3 p, vec3 a, float r) {
  return mix(dot(p,a)*a,p,cos(r)) + sin(r)*cross(p,a);
}

void main() {
  vec2 uv = (gl_FragCoord.xy / RENDERSIZE.xy) - 0.5;
  uv.x *= RENDERSIZE.x / RENDERSIZE.y;
  uv *= 2.0 / Zoom;

  float t = TIME * Speed;

  // Second shader's camera (dominant)
  float yaw = 0.0;
  float pitch = 0.0;
  float roll = 0.0;

  yaw = CameraOrbit;
  pitch = CameraPitch;
  roll = CameraRoll;

  // Camera position and direction
  vec3 camTarget = vec3(0.0);
  vec3 camOffset = vec3(
    sin(yaw) * cos(pitch),
    sin(pitch),
    cos(yaw) * cos(pitch)
  ) * (4.0 - Morph * 3.0);

  // Add shake from second shader
  float shakeX = sin(t * 13.0) * ShakeAmount;
  float shakeY = cos(t * 17.0) * ShakeAmount;

  vec3 camPos = camOffset + vec3(shakeX, shakeY, 0.0);

  vec3 fwd = normalize(camTarget - camPos);
  vec3 right = normalize(cross(vec3(0.0,1.0,0.0), fwd));
  vec3 up = cross(fwd, right);
  vec3 d = normalize(fwd + uv.x * right + uv.y * up);

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

  int fractalIters = int(clamp(FractalIterations, 1.0, 10.0));
  int maxSteps = int(clamp(RaymarchSteps, 20.0, MAX_STEPS));

  // *** DECLARE dominance locals (fixed) ***
  float fractalDominance = clamp(FractalDominance, 0.0, 1.0);
  float textureDominance = clamp(TextureDominance, 0.0, 1.0);

  // Warp camera origin position with first shader’s texture warp - subtle, scaled down
  vec3 ro = vec3(0.0, 0.0, -3.0);
  vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
  vec3 roWarped = ro + warp * TextureWarp * 0.3; // reduce warp influence

  for (int i = 0; i < MAX_STEPS; i++) {
    if(i >= maxSteps) break;

    vec3 p = roWarped + dist * d;

    // Apply first shader fractal transforms subtly wrapped into second shader's fractal
    p = applyTransform(p, mode, chaos * 0.5, sym, chspd);

    // Original fractal distance
    float fractalDist = scene(p, geo, chaos, chaosMix);

    // Texture-based displacement (from first shader)
    float textureInfluence = textureDominance; // use local var

    // Modify the distance by mixing fractal and texture influence
    float dVal = mix(
        fractalDist,                             // fractal shape distance
        fractalDist * (1.0 - textureInfluence) * 0.7,  // slightly reduce fractal shape by texture dominance
        fractalDominance                          // control fractal dominance
    );

    // Fractal iterations from second shader
    float s = 2.0;
    float e = 0.0;
    vec3 q = p;
    for(int j = 0; j < 10; j++) {
      if(j >= fractalIters) break;
      e = 2.0 / min(dot(q,q), 1.0);
      q = abs(q)*e - vec3(3.0, 20.0, 9.0 + Morph * 4.0);
      s *= e;
    }

    dVal += abs(length(q.xy - clamp(q.xy, -0.5, 0.5)) / s) + 0.005;

    dVal = max(abs(dVal), 0.01);

    float fade = exp(-float(i)*0.03*sharp);
    float focus = smoothstep(FocusNear, FocusFar, dist);

    // Color palettes mixed — dominant second shader palette with first shader subtle texture color
    vec3 palCol = getColorPalette(pal, p.z + t * 0.1);
    vec3 texCol = triplanarTexture(p * TextureScale, 1.0);

    // Pulsing effect from second shader
    float pulse = sin(TIME * ChaosSpeed) * 0.5 + 0.5;
    float pulse2 = sin(float(i) * PulseRate + t) * PulseStrength;

    vec3 baseCol = mix(palCol, texCol, textureDominance); // use local var
    vec3 pulseCol = mix(vec3(1.0), palette(log(s)*0.5 + pulse2, ColorPaletteMode), 0.3);

    // Mix fractal-driven pulse color and texture color by fractal dominance
    vec3 finalCol = mix(baseCol, pulseCol, fractalDominance);

    // Use finalCol in accumulation
    col += finalCol * (0.005 / (0.01 + dVal * falloff)) * fade * focus;

    dist += dVal;
    if(dist > BAILOUT) break;
  }

  // Final color correction with glow, brightness, contrast, saturation
  col = adjust(pow(col, vec3(4.0)), br * glow, Saturation, ct);

  // Glitch effect from second shader
  if(Glitch > 0.0) {
    float n = glitchNoise(gl_FragCoord.xy) * Glitch;
    col += n * vec3(1.0, 0.2, 0.0);
  }

  gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
