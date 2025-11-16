/*
{
    "CATEGORIES": [
        "Fractal",
        "Volumetric",
        "Psychedelic"
    ],
    "DESCRIPTION": "A modified version of the original shader, incorporating triplanar texture mapping, ray origin warping, and camera focus from the reference to create a more dynamic and controlled volumetric effect. The core geometry and color logic remain from the original.",
    "ISFVSN": "2.0",
    "INPUTS": [
        { "NAME": "Speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0, "LABEL": "Primary Speed" },
        { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0, "LABEL": "Global Zoom" },
        { "NAME": "GeometryMode", "TYPE": "float", "DEFAULT": 2.0, "MIN": 0.0, "MAX": 3.0, "LABEL": "Geometry Mode", "ANNOTATIONS": { "0.0": "Box", "1.0": "Sphere", "2.0": "Menger Sponge", "3.0": "Twist Fractal" } },
        { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0, "LABEL": "Transform Mode", "ANNOTATIONS": { "0.0": "None", "1.0": "Kaleido", "2.0": "Abs" } },
        { "NAME": "KaleidoSymmetry", "TYPE": "float", "DEFAULT": 4.0, "MIN": 1.0, "MAX": 12.0, "LABEL": "Kaleido Symmetry" },
        { "NAME": "FractalIterations", "TYPE": "float", "DEFAULT": 4.0, "MIN": 1.0, "MAX": 8.0, "LABEL": "Fractal Iterations" },
        { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0, "LABEL": "Chaos Strength" },
        { "NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0, "LABEL": "Chaos Color Mix" },
        { "NAME": "TextureMode", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0, "LABEL": "Texture Warping", "ANNOTATIONS": { "0.0": "Off", "1.0": "On" } },
        { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0, "LABEL": "Texture Warp Intensity" },
        { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0, "LABEL": "Texture Scale" },
        { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 2, "MIN": 0, "MAX": 19, "LABEL": "Palette Mode" },
        { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0, "MAX": 3.0, "LABEL": "Global Brightness" },
        { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 3.0, "LABEL": "Global Contrast" },
        { "NAME": "Glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0, "LABEL": "Global Glow" },
        { "NAME": "FalloffCurve", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.1, "MAX": 3.0, "LABEL": "Falloff Curve" },
        { "NAME": "Sharpness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0, "LABEL": "Sharpness" },
        { "NAME": "FocusNear", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0, "LABEL": "Camera Focus Near" },
        { "NAME": "FocusFar", "TYPE": "float", "DEFAULT": 2.6, "MIN": 0.1, "MAX": 10.0, "LABEL": "Camera Focus Far" },
        { "NAME": "StepCount", "TYPE": "float", "DEFAULT": 64.0, "MIN": 1.0, "MAX": 128.0, "LABEL": "Raymarch Steps" },
        { "NAME": "CameraOrbit", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "LABEL": "Camera Orbit" },
        { "NAME": "CameraPitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.57, "MAX": 1.57, "LABEL": "Camera Pitch" },
        { "NAME": "CameraRoll", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "LABEL": "Camera Roll" },
        { "NAME": "TextureInput", "TYPE": "image" }
    ]
}
*/

#define BAILOUT 16.0
#define PI 3.14159265359
#define TAU (2.0 * PI)

// Rotates a 2D point
mat2 rot2D(float a) {
    float s = sin(a);
    float c = cos(a);
    return mat2(c, s, -s, c);
}

// Camera matrix for controlling viewpoint
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

// A more robust palette function
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

// Triplanar texture mapping from the reference shader
vec3 triplanarTexture(vec3 p, float scale) {
  vec3 blend = normalize(abs(p));
  blend = pow(blend, vec3(4.0));
  blend /= dot(blend, vec3(1.0));
  vec3 tex_coord = p * scale;

  vec3 tx = texture2D(TextureInput, tex_coord.zy).rgb;
  vec3 ty = texture2D(TextureInput, tex_coord.xz).rgb;
  vec3 tz = texture2D(TextureInput, tex_coord.xy).rgb;

  return tx * blend.x + ty * blend.y + tz * blend.z;
}

// --- Signed Distance Field (SDF) Functions ---

// Box SDF
float sdBox(vec3 p, vec3 b) {
  vec3 q = abs(p) - b;
  return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Menger Sponge Fractal SDF
float sdMenger(vec3 p, float iter) {
  float d = sdBox(p, vec3(1.0));
  float s = 1.0;
  for (int i = 0; i < int(iter); i++) {
    vec3 a = mod(p * s, 2.0) - 1.0;
    s *= 3.0;
    vec3 r = abs(1.0 - 3.0 * abs(a));
    float da = max(r.x, r.y);
    float db = max(r.y, r.z);
    float dc = max(r.z, r.x);
    d = max(d, -1.0/s * max(da, max(db, dc)));
  }
  return d;
}

// Twist Fractal SDF
float sdTwistFractal(vec3 p, float iter) {
  vec3 z = p;
  float d = 1e20;
  for (int i = 0; i < int(iter); i++) {
    z = abs(z) / dot(z,z) - 0.5;
    z.xy *= rot2D(z.z * 5.0 + TIME * 0.5);
    d = min(d, length(z));
  }
  return d;
}

// --- Core Logic ---

// The core mapping function that applies transformations and chaos
float map(vec3 p, float time) {
    // Apply Transform Mode to the base position
    if (TransformMode > 0.5 && TransformMode < 1.5) { // Mode 1: Kaleido
        float angle = atan(p.y, p.x);
        float radius = length(p.xy);
        angle = mod(angle, TAU / KaleidoSymmetry);
        angle = abs(angle - TAU / (2.0 * KaleidoSymmetry));
        p.xy = vec2(cos(angle), sin(angle)) * radius;
    } else if (TransformMode > 1.5) { // Mode 2: Abs
        p = abs(p);
    }
    
    // Apply geometry based on the selected mode
    float dist;
    if (GeometryMode < 0.5) { // Mode 0: Box
        dist = sdBox(p, vec3(1.0));
    } else if (GeometryMode < 1.5) { // Mode 1: Sphere
        dist = length(p) - 1.0;
    } else if (GeometryMode < 2.5) { // Mode 2: Menger Sponge
        dist = sdMenger(p, FractalIterations);
    } else { // Mode 3: Twist Fractal
        dist = sdTwistFractal(p, FractalIterations);
    }
    
    return dist;
}


// Main function to render the scene
void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
    uv *= Zoom;
    float time = TIME * Speed;
    
    vec3 ro = vec3(0.0, 0.0, -3.0);
    vec3 rd = normalize(vec3(uv, 1.0));
    rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

    // Apply texture warping to the ray origin (modification from reference)
    if (TextureMode > 0.5) {
        vec3 textureWarp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
        ro += textureWarp * TextureWarp;
    }
    
    vec3 finalColor = vec3(0.0);
    float dist = 0.0;
    float accumulatedAlpha = 0.0;
    
    // Raymarching loop with a controllable number of steps
    for (int i = 0; i < int(StepCount); i++) {
        vec3 p = ro + rd * dist;

        // Apply chaos to the position at each step (from original shader)
        float chaos_val = sin(p.x * 3.0 + time) + cos(p.y * 2.5 + time) + sin(p.z * 3.5 + time);
        vec3 chaos_offset = vec3(sin(chaos_val * 10.0), cos(chaos_val * 8.0), sin(chaos_val * 7.0)) * ChaosIntensity;
        p += chaos_offset;

        float d = map(p, time);
        
        // Calculate volumetric density and focus factor (modifications from reference)
        float b = 0.005 / (0.01 + d * FalloffCurve);
        float fade = exp(-float(i) * 0.03 * Sharpness);
        float focus = smoothstep(FocusNear, FocusFar, dist);

        // Accumulate color
        if (b > 0.0) {
            vec3 palCol = getColorPalette(int(ColorPaletteMode), length(p) + time);

            // Corrected ChaosMix: Blend a chaotic color effect into the palette color
            float chaos_noise = sin(p.x * 5.0 + time * 3.0) + cos(p.y * 6.0 + time * 2.0) + sin(p.z * 4.0 + time * 1.5);
            vec3 chaosColor = pal(chaos_noise, vec3(0.5), vec3(0.5), vec3(1.0), vec3(0.33, 0.67, 0.0));
            palCol = mix(palCol, chaosColor, ChaosMix);
            
            finalColor += palCol * b * fade * focus;
        }

        // Advance the raymarching distance
        dist += d;
        
        // Exit condition
        if (dist > BAILOUT) break;
    }

    // Final color correction
    finalColor = (finalColor - 0.5) * Contrast + 0.5;
    finalColor *= Brightness;
    finalColor += Glow * (vec3(1.0) - finalColor) * finalColor;
    
    gl_FragColor = vec4(clamp(finalColor, 0.0, 1.0), 1.0);
}
