/*
{
    "CATEGORIES": [
        "Generator",
        "Fractal",
        "Tunnel",
        "Volumetric",
        "Psychedelic"
    ],
    "CREDIT": "Converted from ShaderToy by phreax (2023) + merged features by Unity Shader Expert",
    "DESCRIPTION": "This is a single, unified shader where the tunnel fractal acts as the core animator and driver for the volumetric fractal's geometry and transformations. It is no longer a simple switch but a true combination.",
    "ISFVSN": "2",
    "INPUTS": [
        { "NAME": "Speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0 },
        { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0 },
        { "NAME": "Morph", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "ColorPulse", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 5.0 },
        { "NAME": "PulseSpeed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0 },
        { "NAME": "PulsePattern", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 3.0 },
        { "NAME": "ShakeAmount", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "GlitchAmount", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "PaletteIndex", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 6.0 },
        { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "Saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "XYControl", "TYPE": "point2D", "DEFAULT": [0.5, 0.5] },
        { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0.0, "MAX": 5.0 },
        { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 7.0 },
        { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0 },
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
        { "NAME": "StepCount", "TYPE": "float", "DEFAULT": 6.0, "MIN": 1.0, "MAX": 60.0 },
        { "NAME": "Texture", "TYPE": "image" },
        { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0 }
    ]
}
*/

#ifdef GL_ES
precision highp float;
#endif

#define MAX_STEPS 48
#define BAILOUT 16.0
#define PI 3.141592
#define PHI 1.61803
#define SIN(x) (.5+.5*sin(x))
#define R(p,a,r) mix(a*dot(p,a),p,cos(r))+sin(r)*cross(p,a)

// --- Utility Functions ---
vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
  return a + b * cos(6.2831 * (c * t + d));
}

mat2 rot(float a) { return mat2(cos(a), -sin(a), sin(a), cos(a)); }

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

// --- Color Palette and Post-processing ---
vec3 getColorPalette(int mode, float t) {
  if (mode < 20) {
    return pal(t,
      vec3(0.5 + 0.4*sin(float(mode)*0.5), 0.6 + 0.3*cos(float(mode)*1.2), 0.4 + 0.5*sin(float(mode)*0.9)),
      vec3(0.4),
      vec3(1.0,1.3,0.7),
      vec3(0.1,0.2,0.3)
    );
  }
  return pal(t, vec3(0.5), vec3(0.5), vec3(1.0), vec3(0.0));
}

vec3 applyPost(vec3 c) {
  c = mix(vec3(0.5), c, Contrast);
  float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
  c = mix(vec3(l), c, Saturation);
  return clamp(c * Brightness, 0.0, 1.0);
}

// --- Volumetric Fractal Functions (now driven by the tunnel) ---
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
    if (i >= int(StepCount)) break;
    // THIS IS THE ONLY CHANGE.
    // We add a max() to the denominator to prevent a division by a value too close to zero.
    p = abs(p) / max(dot(p, p + 0.001), 0.00001) - 0.5;
    p *= 0.95;
    d += length(p);
  }
  return d / 20.0;
}

float shapeChaos(vec3 p, float chaos) {
  return (sin(p.x*3. + TIME*ChaosSpeed) + sin(p.y*4. + TIME*ChaosSpeed*1.2) + sin(p.z*5. + TIME*ChaosSpeed*0.8)) * chaos;
}

float sceneVolumetric(vec3 p, int geo, float chaos, float mixAmt) {
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

// --- Tunnel Fractal Logic (used as a driver) ---
vec2 path(float z) {
  return vec2(cos(z / 8.0) * sin(z / 12.0) * 12.0, 0.0);
}

float mapTunnel(vec3 p) {
  vec4 q = vec4(p, 1.0 + Morph * 4.0);
  q.x += 1.0;
  for (int i = 0; i < 6; i++) {
    q.xyz = -1.0 + 2.0 * fract(0.5 + 0.5 * q.xyz);
    q = (1.2 + Morph) * q / max(dot(q.xyz, q.xyz), 0.1);
  }
  vec2 tun = abs(p.xy - path(p.z)) * vec2(0.6, 0.5);
  return min(0.25 * abs(q.y) / q.w, 1.0 - max(tun.x, tun.y));
}


// --- Main Function ---
void main() {
  vec2 uv = -1.0 + 2.0 * (gl_FragCoord.xy / RENDERSIZE.xy);
  uv.x *= RENDERSIZE.x / RENDERSIZE.y;
  uv += (XYControl - 0.5) * 2.0;
  float t = TIME * Speed;
  int geo = int(GeometryType);
    
  vec3 ro = vec3(0.0, 0.0, -3.0);
  vec3 rd = normalize(vec3(uv * FOV, 1.0));
  rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

  vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
  vec3 roWarped = ro + warp * TextureWarp;

  vec3 col = vec3(0.0);
  float dist = 0.0;
    
  // This is the core logic. The tunnel's chaos now drives the volumetric fractal.
  // We use the ray position to calculate a new chaotic vector based on the tunnel's SDF.
  float tunnelDist = mapTunnel(ro + dist * rd);
  vec3 tunnelWarp = vec3(sin(tunnelDist * 50.0 + t), cos(tunnelDist * 40.0 + t), sin(tunnelDist * 60.0 + t)) * Morph * 0.5;

  int mode = int(TransformMode);
  float chaos = ChaosIntensity + tunnelDist * Morph * 2.0;
  float chaosMix = ChaosMix;
  float sym = Symmetry;
  float chspd = ChaosSpeed;
  float sharp = Sharpness;
  float falloff = FalloffCurve;

  for (int i = 0; i < MAX_STEPS; i++) {
    vec3 p = roWarped + dist * rd;
    p += tunnelWarp; // Apply the tunnel warp here

    p = applyTransform(p, mode, chaos, sym, chspd);
    float d = sceneVolumetric(p, geo, chaos, chaosMix);
    d = max(abs(d), 0.01);

    float fade = exp(-float(i)*0.03*sharp);
    float focus = smoothstep(FocusNear, FocusFar, dist);

    vec3 palCol = getColorPalette(int(PaletteIndex), p.z + t * 0.1);
    vec3 texCol = triplanarTexture(p * TextureScale, 1.0);
    float b = 0.005 / (0.01 + d * falloff);

    col += mix(palCol, texCol, 0.5) * b * fade * focus;
    dist += d;
    if (dist > BAILOUT) break;
  }
    
  vec3 finalColor = col * Glow;
  finalColor = 1.0 - exp(-0.5 * finalColor);

  // Post-processing
  finalColor.rgb += vec3(
    sin(TIME * 30.0 + uv.y * 100.0) * GlitchAmount * 0.05,
    cos(TIME * 25.0 + uv.x * 200.0) * GlitchAmount * 0.05,
    sin(TIME * 15.0) * GlitchAmount * 0.03
  );
    
  gl_FragColor = vec4(applyPost(finalColor), 1.0);
}
