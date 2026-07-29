/*
{
    "DESCRIPTION": "A complete synthesis of a 3D raymarching fractal and a hexagonal grid shader. The hexagonal 'flower of life' animation now dynamically warps the fractal geometry for a true hybrid visual effect.",
    "CATEGORIES": ["Fractal", "Volumetric", "Psychedelic", "Optimized", "Generative", "Hexagonal Grid"],
    "ISFVSN": "2.0",
    "INPUTS": [
        { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0, "COMMENT": "Base speed of the fractal's overall movement." },
        { "NAME": "displace", "TYPE": "float", "DEFAULT": 0.04, "MIN": 0.01, "MAX": 0.1, "COMMENT": "Time displacement between the RGB channels. A key psychedelic control from the original hex grid shader." },
        { "NAME": "HexGridSpeed", "TYPE": "float", "DEFAULT": 0.05, "MIN": 0.01, "MAX": 2.2, "COMMENT": "Speed of the hexagonal grid animation." },
        { "NAME": "HexGridDisplace", "TYPE": "float", "DEFAULT": 0.04, "MIN": 0.01, "MAX": 0.2, "COMMENT": "Intensity of the hexagonal grid's geometric displacement." },
        { "NAME": "HexGridSize", "TYPE": "float", "DEFAULT": 36.0, "MIN": 10.0, "MAX": 100.0, "COMMENT": "Scale of the hexagonal grid pattern." },
        { "NAME": "HexGridWave", "TYPE": "float", "DEFAULT": 5.0, "MIN": 1.0, "MAX": 10.0, "COMMENT": "Frequency of the wave pattern within the hex grid." },
        { "NAME": "FractalZoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0 },
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
        { "NAME": "pseudoAudioLevel", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0, "COMMENT": "Audio influence for color pulsing and movement." },
        { "NAME": "shake", "TYPE": "float", "DEFAULT": 0.05, "MIN": 0.0, "MAX": 0.2, "COMMENT": "Amount of screen shake from the hex grid shader." },
        { "NAME": "shimmer", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0, "COMMENT": "Adds a high-frequency noise/shimmer effect from the hex grid shader." },
        { "NAME": "TextureInfluence", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0, "COMMENT": "Mix between procedural color and texture color." }
    ]
}
*/

#define MAX_STEPS 48
#define BAILOUT 16.0
#define PI 3.14159
#define TWO_PI 6.2831853072

// --- Original Fractal Shader Functions ---
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

float shapeSpikeFractal(vec3 p, float time_val) {
    float d = 0.0;
    for (int i = 0; i < 128; i++) {
        if (i >= int(StepCount)) break;
        p = abs(p) / dot(p, p + 0.001) - 0.5;
        p *= 0.95;
        d += length(p);
    }
    return d / 20.0;
}

float shapeChaos(vec3 p, float chaos, float time_val) {
    return (sin(p.x*3. + time_val*ChaosSpeed) + sin(p.y*4. + time_val*ChaosSpeed*1.2) + sin(p.z*5. + time_val*ChaosSpeed*0.8)) * chaos;
}

float scene(vec3 p, int geo, float chaos, float mixAmt, float time_val) {
    float base;
    if (geo == 0) base = length(p) - 1.0;
    else if (geo == 1) {
        vec2 q = vec2(length(p.xz)-1.0, p.y);
        base = length(q) - 0.3;
    }
    else if (geo == 2) base = shapeSpikeFractal(p * 1.2, time_val);
    else base = shapeSpikeFractal(p, time_val);
    return mix(base, shapeChaos(p, chaos, time_val), mixAmt);
}

vec3 applyTransform(vec3 p, int mode, float chaos, float sym, float chspd, float time_val) {
    p *= max(sym, 0.001);
    if (mode == 1) p = abs(p);
    else if (mode == 2) p += sin(p * 3.0 + time_val * chspd) * chaos * 0.3;
    else if (mode == 3) {
        p += sin(p * (1.0 + chaos * 2.0) + time_val * chspd) * chaos * 0.5;
        p = fract(p * 1.5) - 0.75;
    }
    if (mode == 4 || mode == 5) {
        float a = atan(p.z, p.x);
        float r = length(p.xz);
        float spin = time_val * chspd * (mode == 4 ? 0.2 : 0.3);
        a += spin;
        p.x = cos(a) * r;
        p.z = sin(a) * r;
    }
    return p;
}

// --- New Hexagonal Grid Functions (from the second shader) ---
vec2 rotate(vec2 v, float a) {
    float c = cos(a), s = sin(a);
    return mat2(c, -s, s, c) * v;
}

vec3 coordToHex(vec2 coord, float scale, float angle) {
    vec2 c = rotate(coord, angle);
    float q = (1.0 / 3.0 * sqrt(3.0) * c.x - 1.0 / 3.0 * c.y) * scale;
    float r = 2.0 / 3.0 * c.y * scale;
    return vec3(q, r, -q - r);
}

vec3 hexToCell(vec3 hex, float m) {
    return fract(hex / m) * 2.0 - 1.0;
}

float absMax(vec3 v) {
    return max(max(abs(v.x), abs(v.y)), abs(v.z));
}

float nsin(float v) {
    return 0.5 + 0.5 * sin(v * TWO_PI);
}

float hexToFloat(vec3 hex, float amt) {
    return mix(absMax(hex), 1.0 - length(hex) / sqrt(3.0), amt);
}

// This function now calculates the distortion value based on the hex grid
float getHexGridDistortion(vec2 tx, float time_val) {
    float level = pseudoAudioLevel;
    float angle = PI * nsin(time_val * 0.1) + PI / 6.0;
    float len = 2.0 / 122.0 * level + 1.0;
    float value = time_val * 0.005 + level * 0.000752;

    vec3 hex = coordToHex(tx, HexGridSize * nsin(time_val * 0.01), angle);
    for (int i = 0; i < 3; i++) {
        float off = float(i) / 3.0;
        vec3 cell = hexToCell(hex, 1.0 + float(i));
        float wt = mod(time_val, 10.0);
        value += nsin(hexToFloat(cell, nsin(len + wt + off)) * HexGridWave * nsin(wt * 0.5 + off) + len + wt);
    }
    return clamp(0.5 + 0.5 * sin(value), 0.0, 1.0);
}


// The complete raymarching pass, now with integrated distortion
vec3 renderFractalPass(vec3 ro, vec3 rd, float time_val, float distortion_val) {
    vec3 col = vec3(0.0);
    float dist = 0.0;

    int mode = int(TransformMode);
    int geo = int(GeometryType);
    float chaos = ChaosIntensity;
    float chaosMix = ChaosMix;
    float sym = Symmetry;
    float chspd = ChaosSpeed;
    float sharp = Sharpness;
    float falloff = FalloffCurve;
    int pal_id = int(ColorPaletteMode);
    float tex_influence = TextureInfluence;

    for (int i = 0; i < MAX_STEPS; i++) {
        vec3 p = ro + dist * rd;
        
        // APPLY THE HEXAGONAL GRID DISTORTION HERE
        float hex_distort = distortion_val * HexGridDisplace;
        vec3 hex_offset = vec3(sin(p.x*10.0)*hex_distort, cos(p.y*10.0)*hex_distort, sin(p.z*10.0)*hex_distort);
        p += hex_offset;

        p = applyTransform(p, mode, chaos, sym, chspd, time_val);
        float d = scene(p, geo, chaos, chaosMix, time_val);
        d = max(abs(d), 0.01);

        float fade = exp(-float(i) * 0.03 * sharp);
        float focus = smoothstep(FocusNear, FocusFar, dist);

        vec3 palCol = getColorPalette(pal_id, p.z + time_val * 0.1);
        vec3 texCol = triplanarTexture(p * TextureScale, 1.0);
        float b = 0.005 / (0.01 + d * falloff);

        col += mix(palCol, texCol, tex_influence) * b * fade * focus;
        dist += d;
        if (dist > BAILOUT) break;
    }
    return col;
}

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
    // Apply shake from the hex shader
    uv += sin(TIME * ChaosSpeed * 0.2) * shake;
    uv *= FOV;

    vec3 ro = vec3(0.0, 0.0, -3.0);
    vec3 rd = normalize(vec3(uv * FractalZoom, 1.0));
    rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

    vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
    vec3 roWarped = ro + warp * TextureWarp;

    // Time is now controlled by HexGridSpeed for the hex grid animation
    float t_base = TIME * Speed * (1.0 + pseudoAudioLevel * 0.5);
    float hex_t = TIME * HexGridSpeed * (1.0 + pseudoAudioLevel * 0.5);
    
    // Corrected variable usage
    float timeDisplace = displace;

    vec3 finalColor = vec3(0.0);
    
    // --- The new, corrected integration ---
    // We now have two independent animation timings: one for the fractal, one for the hex grid
    // Generate the hex grid distortion value based on a time-displaced UV coordinate
    float hex_distortion_r = getHexGridDistortion(uv, hex_t);
    float hex_distortion_g = getHexGridDistortion(uv, hex_t + timeDisplace);
    float hex_distortion_b = getHexGridDistortion(uv, hex_t + 2.0 * timeDisplace);

    // Call the render pass three times, each with its own distortion value
    finalColor.r = renderFractalPass(roWarped, rd, t_base, hex_distortion_r).r;
    finalColor.g = renderFractalPass(roWarped, rd, t_base + timeDisplace, hex_distortion_g).g;
    finalColor.b = renderFractalPass(roWarped, rd, t_base + 2.0 * timeDisplace, hex_distortion_b).b;

    // Apply post-processing from both shaders
    float pulse = sin(TIME * ChaosSpeed) * 0.5 + 0.5;
    finalColor *= 1.0 + 0.3 * pulse;
    
    finalColor += shimmer * vec3(sin(uv.x * 30.0 + TIME), sin(uv.y * 30.0 + TIME), sin((uv.x + uv.y) * 15.0 + TIME));
    
    finalColor = (finalColor - 0.5) * Contrast + 0.5;
    finalColor *= Brightness * Glow;
    
    gl_FragColor = vec4(clamp(finalColor, 0.0, 1.0), 1.0);
}
