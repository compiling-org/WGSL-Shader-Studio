/*
{
    "DESCRIPTION": "Optimized fractal with visible texture influence and reduced GPU load.",
    "CATEGORIES": ["Fractal", "Optimized", "Volumetric"],
    "ISFVSN": "2.0",
    "INPUTS": [
        { "NAME": "PulseSpeed", "TYPE": "float", "DEFAULT": 2.0, "MIN": 0.1, "MAX": 10.0, "LABEL": "Tunnel Speed" },
        { "NAME": "MorphRate", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0, "LABEL": "Fractal Morph Rate" },
        { "NAME": "TwistAmount", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0.0, "MAX": 3.0, "LABEL": "Tunnel Twist" },
        { "NAME": "GridFrequency", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.1, "MAX": 10.0, "LABEL": "Grid Pattern Frequency" },
        { "NAME": "DistortionStrength", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0, "LABEL": "Fractal Distortion" },
        { "NAME": "GlowBoost", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.1, "MAX": 5.0, "LABEL": "Glow Multiplier" },
        { "NAME": "FogDensity", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 2.0, "LABEL": "Fog Density" },
        { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5, "LABEL": "Space Transform" },
        { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0, "LABEL": "Chaos Intensity" },
        { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0, "LABEL": "Chaos Speed" },
        { "NAME": "Symmetry", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 4.0, "LABEL": "Symmetry Scale" },
        { "NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0, "LABEL": "Chaos Blend" },
        { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6, "LABEL": "Base Geometry", "VALUES": ["Sphere", "Torus", "Spike Fractal", "Grid Fractal", "Liminal Fractal", "Mixed Liminal", "Mixed Spike"] },
        { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0, "LABEL": "Zoom Level" },
        { "NAME": "FOV", "TYPE": "float", "DEFAULT": 1.6, "MIN": 0.2, "MAX": 3.0, "LABEL": "Field of View" },
        { "NAME": "CameraOrbit", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "LABEL": "Camera Orbit" },
        { "NAME": "CameraPitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.57, "MAX": 1.57, "LABEL": "Camera Pitch" },
        { "NAME": "CameraRoll", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "LABEL": "Camera Roll" },
        { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 19, "MIN": 0, "MAX": 19, "LABEL": "Palette Mode" },
        { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0, "MAX": 3.0, "LABEL": "Global Brightness" },
        { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 3.0, "LABEL": "Global Contrast" },
        { "NAME": "GlowFalloff", "TYPE": "float", "DEFAULT": 30.0, "MIN": 1.0, "MAX": 200.0, "LABEL": "Glow Falloff" },
        { "NAME": "Sharpness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0, "LABEL": "Raymarch Sharpness" },
        { "NAME": "FalloffCurve", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.1, "MAX": 3.0, "LABEL": "Color Falloff" },
        { "NAME": "StepCount", "TYPE": "float", "DEFAULT": 6, "MIN": 1, "MAX": 60, "LABEL": "Raymarch Steps" },
        { "NAME": "FocusNear", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0, "LABEL": "Focus Near" },
        { "NAME": "FocusFar", "TYPE": "float", "DEFAULT": 2.6, "MIN": 0.1, "MAX": 10.0, "LABEL": "Focus Far" },
        { "NAME": "Texture", "TYPE": "image" },
        { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0, "LABEL": "Ray Origin Warp" },
        { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0, "LABEL": "Texture Scale" }
    ]
}
*/

#define PI 3.14159
#define BAILOUT 16.0

// A combination of both shader's rotation and transformation matrices.
mat2 rot(float a) {
    float c = cos(a), s = sin(a);
    return mat2(c, s, -s, c);
}

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

vec3 applyTransform(vec3 p, float mode, float chaos, float sym, float chspd) {
    p *= max(sym, 0.001);
    if (mode < 1.5) p = abs(p);
    else if (mode < 2.5) p += sin(p * 3.0 + TIME * chspd) * chaos * 0.3;
    else if (mode < 3.5) {
        p += sin(p * (1.0 + chaos * 2.0) + TIME * chspd) * chaos * 0.5;
        p = fract(p * 1.5) - 0.75;
    }
    if (mode > 3.5 && mode < 5.5) {
        float a = atan(p.z, p.x);
        float r = length(p.xz);
        float spin = TIME * chspd * (mode < 4.5 ? 0.2 : 0.3);
        a += spin;
        p.x = cos(a) * r;
        p.z = sin(a) * r;
    }
    return p;
}

// All SDFs are included and combined into a single function.
float shapeSpikeFractal(vec3 p) {
    float d = 0.0;
    for (int i = 0; i < 128; i++) {
        if (i >= int(StepCount)) break;
        p = abs(p) / dot(p, p + 0.001) - 0.5;
        p *= 0.95;
        d += length(p);
    }
    return d / 20.0;
}

float liminalFractal(vec3 p, float t) {
    float m = 100.0;
    for (int i = 0; i < 7; i++) {
        p = abs(p) / dot(p, p) - MorphRate;
        p.xy *= rot(t * 0.05 + float(i) * 0.1);
        m = min(m, length(p));
    }
    return m;
}

float shapeChaos(vec3 p, float chaos) {
    return (sin(p.x * 3.0 + TIME * ChaosSpeed) + sin(p.y * 4.0 + TIME * ChaosSpeed * 1.2) + sin(p.z * 5.0 + TIME * ChaosSpeed * 0.8)) * chaos;
}

float sdSphere(vec3 p, float r) { return length(p) - r; }
float sdTorus(vec3 p, vec2 t) { vec2 q = vec2(length(p.xz) - t.x, p.y); return length(q) - t.y; }
float sdGrid(vec3 p, float freq) { return sin(p.x * freq) * sin(p.y * freq) * sin(p.z * freq); }

// A new, fully featured scene function that incorporates all geometries.
float scene(vec3 p, float t, float geo, float chaos, float mixAmt, float distStrength) {
    float base;
    vec3 distorted_p = p + sin(p.yzx + t * 0.5) * distStrength;

    if (geo < 0.5) base = sdSphere(distorted_p, 1.0);
    else if (geo < 1.5) base = sdTorus(distorted_p, vec2(1.0, 0.3));
    else if (geo < 2.5) base = shapeSpikeFractal(distorted_p);
    else if (geo < 3.5) base = sdGrid(distorted_p, GridFrequency);
    else if (geo < 4.5) base = liminalFractal(distorted_p, t);
    else if (geo < 5.5) base = mix(liminalFractal(distorted_p, t), sdGrid(distorted_p, GridFrequency) * 0.1, 0.5);
    else base = mix(shapeSpikeFractal(distorted_p), sdSphere(distorted_p, 1.0), 0.5);
    
    return mix(base, shapeChaos(p, chaos), mixAmt);
}

// Both palette systems are now combined into one.
vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
    return a + b * cos(6.2831 * (c * t + d));
}

vec3 getLiminalPalette(float t, float id) {
    if (id < 1.0) return 0.5 + 0.5 * cos(6.2831 * (t + vec3(0.0, 0.33, 0.67)));
    if (id < 2.0) return vec3(sin(t * 3.0), sin(t * 2.5 + 1.0), sin(t * 4.0 + 2.0)) * 1.1;
    if (id < 3.0) return vec3(1.0 - abs(sin(t * 3.0 + vec3(0.5, 0.3, 0.1)))) * 1.2;
    if (id < 4.0) return vec3(0.3 + 0.4 * sin(t + vec3(1, 2, 3)));
    if (id < 5.0) return vec3(sin(t * 7.0), sin(t * 13.0), sin(t * 17.0));
    if (id < 6.0) return vec3(1.0, 0.7 + 0.3 * sin(t * 3.5), 0.6 * sin(t * 2.0));
    if (id < 7.0) return vec3(exp(-t * 2.0)) * vec3(1.2, 0.8, 1.5);
    return 0.5 + 0.5 * cos(6.2831 * t + vec3(0.0, 0.6, 1.2));
}

vec3 getColorPalette(float mode, float t) {
    if (mode < 8.0) return getLiminalPalette(t, mode);
    return pal(t,
        vec3(0.5 + 0.4*sin(float(mode)*0.5), 0.6 + 0.3*cos(float(mode)*1.2), 0.4 + 0.5*sin(float(mode)*0.9)),
        vec3(0.4),
        vec3(1.0,1.3,0.7),
        vec3(0.1,0.2,0.3)
    );
}

// Triplanar texturing is now a core part of the raymarching.
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

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
    uv *= FOV;
    float t = TIME * PulseSpeed;
    
    // The starting camera position from the first shader.
    vec3 ro = vec3(0.0, 0.0, -4.0 + 1.5 * sin(t * 0.3));
    vec3 rd = normalize(vec3(uv * Zoom, 1.0));
    
    // The camera matrix from the second shader is now used for control.
    rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

    // The texture warping from the second shader is now applied.
    vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
    vec3 roWarped = ro + warp * TextureWarp;

    vec3 col = vec3(0.0);
    float dist = 0.0;
    float glow = 0.0;

    // All parameters are declared for clarity and easy modification.
    float mode = TransformMode;
    float geo = GeometryType;
    float chaos = ChaosIntensity;
    float chaosMix = ChaosMix;
    float sym = Symmetry;
    float chspd = ChaosSpeed;
    float br = Brightness;
    float ct = Contrast;
    float glowBoost = GlowBoost;
    float palMode = ColorPaletteMode;
    float sharp = Sharpness;
    float falloff = FalloffCurve;
    float distStr = DistortionStrength;

    // The raymarching loop is a complete blend of both shaders.
    for (int i = 0; i < int(StepCount); i++) {
        vec3 p = roWarped + dist * rd;
        
        // The twist and transform logic from both shaders is now applied here.
        p.xy *= rot(t * 0.15 + sin(p.z * 0.1) * TwistAmount);
        p = applyTransform(p, mode, chaos, sym, chspd);
        
        // The SDF is now a complete fusion.
        float d = scene(p, t, geo, chaos, chaosMix, distStr);
        d = max(abs(d), 0.01);

        // All coloring, fading, and focusing logic is now applied here.
        float fade = exp(-float(i) * 0.03 * sharp);
        float focus = smoothstep(FocusNear, FocusFar, dist);
        
        vec3 palCol = getColorPalette(palMode, p.z + t * 0.1);
        vec3 texCol = triplanarTexture(p * TextureScale, 1.0);
        
        // The glow calculation from the first shader.
        float g = sdGrid(p, GridFrequency);
        float combinedGlow = exp(-abs(d - 0.08) * GlowFalloff) + 0.15 * abs(g);
        
        vec3 c = getColorPalette(palMode, combinedGlow + t * 0.2 + p.z * 0.01) * combinedGlow * glowBoost;

        // The color is a mix of the two shaders' coloring methods.
        float b = 0.005 / (0.01 + d * falloff);
        vec3 finalCol = mix(palCol * b, texCol * b, 0.5);
        
        col += mix(finalCol, c * 0.04, 0.5) * fade * focus;

        dist += d;
        if (dist > BAILOUT) break;
    }

    // The final post-processing is a combination of both.
    float pulse = sin(TIME * ChaosSpeed) * 0.5 + 0.5;
    col *= 1.0 + 0.3 * pulse;
    col = (col - 0.5) * ct + 0.5;
    col *= br * glowBoost;

    col = pow(col, vec3(0.6));
    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
