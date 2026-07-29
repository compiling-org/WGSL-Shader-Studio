/*
{
    "DESCRIPTION": "A true synthesis of two shaders, blending a 3D raymarcher with the chaotic 2D fractal logic.",
    "CATEGORIES": ["Generator", "Fractal", "Volumetric", "Psychedelic"],
    "ISFVSN": "2.0",
    "INPUTS": [
       
        { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0 },
        { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0 },
        { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5, "LABELS": ["None", "Absolute", "Chaos", "Fractal", "SpinX", "SpinY"] },
        { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6, "VALUES": ["Sphere", "Torus", "Spike Fractal", "Grid Fractal", "Liminal Fractal", "Mixed Liminal", "Mixed Spike"] },
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
        
        
        { "NAME": "colorPulse", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "shakeAmount", "TYPE": "float", "DEFAULT": 0.01, "MIN": 0.0, "MAX": 0.1 },
        { "NAME": "cameraShift", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
        { "NAME": "fractalMorph", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
        
        
        { "NAME": "LiminalMix", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "LiminalScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0 }
    ]
}
*/

#define MAX_STEPS 48
#define BAILOUT 16.0
#define PI 3.14159

// Reused helper functions from the first shader
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

mat2 rot(float a) {
    float s = sin(a), c = cos(a);
    return mat2(c, s, -s, c);
}

// Advanced palettes from the first shader
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

// Triplanar texturing from the first shader
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

// SDFs from the first shader
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

// The core logic of the second shader, now in 3D
float liminalFractal(vec3 p, float t, float scale) {
    p *= scale;
    float m = 100.0;
    for (int i = 0; i < 7; i++) {
        p = abs(p) / dot(p, p) - fractalMorph;
        p.xy *= rot(t * 0.05 + float(i) * 0.1);
        m = min(m, length(p));
    }
    return m;
}

// New function to apply the chaotic transformations of the 2D shader
vec3 applyLiminalWarp(vec3 p, float t) {
    p.xy *= Zoom + dot(p.xy, p.xy) * 2.0;
    float s = sin(t * 0.1 * 1.585 * PI);
    p.x += s * s * s;
    p.xy *= rot(tan(t * PI * 1.585 * .2) * .1);
    p.xy *= 7.0 - atan(5.0 * cos(t * 2.0 * PI * 1.585 * .25) * PI * 1.585) * 1.5;
    return p;
}


float shapeChaos(vec3 p, float chaos) {
    return (sin(p.x*3. + TIME*ChaosSpeed) + sin(p.y*4. + TIME*ChaosSpeed*1.2) + sin(p.z*5. + TIME*ChaosSpeed*0.8)) * chaos;
}

float sdSphere(vec3 p, float r) { return length(p) - r; }
float sdTorus(vec3 p, vec2 t) { vec2 q = vec2(length(p.xz)-t.x, p.y); return length(q) - t.y; }
float sdGrid(vec3 p, float freq) { return sin(p.x * freq) * sin(p.y * freq) * sin(p.z * freq); }

// Combined scene function, now including the liminal fractal logic
float scene(vec3 p, float t, float geo, float chaos, float mixAmt, float LiminalMix, float liminalScale) {
    float base;
    if (geo < 0.5) base = sdSphere(p, 1.0);
    else if (geo < 1.5) base = sdTorus(p, vec2(1.0, 0.3));
    else if (geo < 2.5) base = shapeSpikeFractal(p);
    else if (geo < 3.5) base = sdGrid(p, 4.0);
    else if (geo < 4.5) base = liminalFractal(p, t, liminalScale);
    else if (geo < 5.5) base = mix(liminalFractal(p, t, liminalScale), sdGrid(p, 4.0) * 0.1, 0.5);
    else base = mix(shapeSpikeFractal(p), sdSphere(p, 1.0), 0.5);

    float finalShape = mix(shapeSpikeFractal(p), liminalFractal(p, t, liminalScale), LiminalMix);
    
    return mix(finalShape, shapeChaos(p, chaos), mixAmt);
}

// Space transformations from the first shader
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

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
    uv += sin(vec2(TIME * 13.1, TIME * 11.5)) * shakeAmount;
    uv.x += cameraShift;
    uv *= FOV;
    
    float t = TIME * Speed;
    
    // Ray origin and direction, with camera controls from the first shader
    vec3 ro = vec3(0.0, 0.0, -3.0);
    vec3 rd = normalize(vec3(uv * Zoom, 1.0));
    rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;
    
    // Texture warping for the ray origin
    vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
    vec3 roWarped = ro + warp * TextureWarp;
    
    vec3 col = vec3(0.0);
    float dist = 0.0;
    
    // Parameters for the loop
    float mode = TransformMode;
    float geo = GeometryType;
    float chaos = ChaosIntensity;
    float chaosMix = ChaosMix;
    float sym = Symmetry;
    float br = Brightness;
    float ct = Contrast;
    float glow = Glow;
    float palMode = ColorPaletteMode;
    float sharp = Sharpness;
    float falloff = FalloffCurve;
    
    // The raymarching loop, now using the combined functions
    for (int i = 0; i < MAX_STEPS; i++) {
        vec3 p = roWarped + dist * rd;
        
        // APPLY THE LIMINAL INFLUENCE HERE
        vec3 warpedP = applyLiminalWarp(p, t);
        p = mix(p, warpedP, LiminalMix);

        p = applyTransform(p, mode, chaos, sym, ChaosSpeed);
        
        float d = scene(p, t, geo, chaos, chaosMix, LiminalMix, LiminalScale);
        d = max(abs(d), 0.01);
        
        float fade = exp(-float(i)*0.03*sharp);
        float focus = smoothstep(FocusNear, FocusFar, dist);
        
        vec3 palCol = getColorPalette(palMode, p.z + t * 0.1) * colorPulse;
        vec3 texCol = triplanarTexture(p * TextureScale, 1.0);
        float b = 0.005 / (0.01 + d * falloff);
        
        col += mix(palCol, texCol, 0.5) * b * fade * focus;
        
        dist += d;
        if (dist > BAILOUT) break;
    }
    
    // Post-processing from both shaders
    float pulse = sin(TIME * ChaosSpeed) * 0.5 + 0.5;
    col *= 1.0 + 0.3 * pulse;
    col = (col - 0.5) * ct + 0.5;
    col *= br * glow;
    
    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
