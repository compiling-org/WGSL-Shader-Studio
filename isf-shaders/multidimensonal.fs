/*{
  "DESCRIPTION": "Fusion: full-featured ISF volumetric fractal — incorporates both reference shaders entirely (all sliders & features). Volumetric raymarch, triplanar texture influence, many palette modes, camera controls, chaos/morph transforms, feedback buffer, and full post controls.",
  "CATEGORIES": ["Fractal","Volumetric","Psychedelic","Technical Art"],
  "ISFVSN": "2.0",
  "INPUTS": [
    { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0 },
    { "NAME": "speed", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.01, "MAX": 2.0 },

    { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0 },
    { "NAME": "zoom", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.1, "MAX": 5.0 },

    { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5 },
    { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6 },

    { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0 },

    { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 19, "MIN": 0, "MAX": 19 },
    { "NAME": "colorPalette", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 3.0, "LABELS": ["Rainbow","Neon","Fire","Ice"] },
    { "NAME": "ColorPulse", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 5.0 },

    { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0, "MAX": 3.0 },
    { "NAME": "brightness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 3.0 },
    { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 3.0 },
    { "NAME": "Saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 3.0 },

    { "NAME": "Glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "Symmetry", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0 },

    { "NAME": "Sharpness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0 },
    { "NAME": "FalloffCurve", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.1, "MAX": 3.0 },

    { "NAME": "Morphing", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "FractalControl", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "Warping", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 5.0 },

    { "NAME": "CameraOrbit", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14 },
    { "NAME": "CameraPitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.57, "MAX": 1.57 },
    { "NAME": "CameraRoll", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14 },

    { "NAME": "FocusNear", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0 },
    { "NAME": "FocusFar", "TYPE": "float", "DEFAULT": 2.6, "MIN": 0.1, "MAX": 10.0 },

    { "NAME": "FOV", "TYPE": "float", "DEFAULT": 1.6, "MIN": 0.2, "MAX": 3.0 },

    { "NAME": "StepCount", "TYPE": "float", "DEFAULT": 6, "MIN": 1, "MAX": 60 },
    { "NAME": "MAX_ITERATIONS", "TYPE": "float", "DEFAULT": 7, "MIN": 1, "MAX": 128 },

    { "NAME": "Texture", "TYPE": "image" },
    { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0 },

    { "NAME": "Tile", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.01, "MAX": 8.0 }
  ],
  "PASSES": [
    {
      "TARGET": "bufferA",
      "PERSISTENT": true
    }
  ]
}*/

#define PI 3.14159265359
#define MAX_STEPS 128
#define BAILOUT 16.0

// ------------------------- Utility math -------------------------
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

mat2 rot2d(float a) {
    return mat2(cos(a), -sin(a), sin(a), cos(a));
}

float hash21(vec2 p) { p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 78.233); return fract(p.x * p.y); }

// ------------------------- Palettes (both refs combined) -------------------------
vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
    return a + b * cos(6.28318530718 * (c * t + d));
}

vec3 paletteRainbow(float t) { return vec3(0.5 + 0.5 * cos(6.28318 * (t + vec3(0.0, 0.33, 0.67)))); }
vec3 paletteNeon(float t)   { return vec3(0.5 + 0.5 * sin(6.28318 * (t + vec3(0.0, 0.2, 0.4)))); }
vec3 paletteFire(float t)   { return vec3(pow(t,0.6), pow(t,0.3), 0.0); }
vec3 paletteIce(float t)    { return vec3(0.0, pow(t,0.5)*0.8, pow(t,0.3)*1.2); }

vec3 getColorPalette(int mode, float t) {
    // first honor the small set from the second shader for modes 0..3
    if(mode == 0) return paletteRainbow(t);
    if(mode == 1) return paletteNeon(t);
    if(mode == 2) return paletteFire(t);
    if(mode == 3) return paletteIce(t);
    // Otherwise use multi-param pal like the first shader; provide several presets up to 19
    // Use mode to vary parameters deterministically
    float m = float(mode);
    vec3 a = vec3(0.5 + 0.4*sin(m*0.5 + 0.1), 0.6 + 0.3*cos(m*1.2 + 0.2), 0.4 + 0.5*sin(m*0.9 - 0.3));
    vec3 b = vec3(0.5, 0.4, 0.35) * (1.0 + 0.02 * m);
    vec3 c = vec3(1.0, 1.3, 0.7) + 0.02 * m;
    vec3 d = vec3(0.1, 0.2, 0.3) + 0.01 * m;
    return pal(t, a, b, c, d);
}

// ------------------------- Texturing -------------------------
vec3 triplanarTexture(vec3 p, float scale) {
    // normalized blend based on world normal approximation (abs p)
    vec3 blend = normalize(abs(p) + 1e-6);
    blend = pow(blend, vec3(4.0));
    blend /= (blend.x + blend.y + blend.z + 1e-6);
    vec2 xz = fract(p.zy * scale * Tile);
    vec2 yz = fract(p.xz * scale * Tile);
    vec2 xy = fract(p.xy * scale * Tile);
    vec3 tx = texture2D(Texture, xz).rgb;
    vec3 ty = texture2D(Texture, yz).rgb;
    vec3 tz = texture2D(Texture, xy).rgb;
    return tx * blend.x + ty * blend.y + tz * blend.z;
}

// ------------------------- Fractal primitives -------------------------
float shapeSpikeFractal(vec3 p, int iters) {
    float d = 0.0;
    for (int i = 0; i < 128; i++) {
        if (i >= iters) break;
        // inspired by original: inversion-ish spikes
        float denom = dot(p, p) + 0.001;
        p = abs(p) / denom - 0.5;
        p *= 0.95;
        d += length(p);
    }
    return d / 20.0;
}

float shapeChaos(vec3 p, float chaos) {
    return (sin(p.x*3.0 + TIME*ChaosSpeed) + sin(p.y*4.0 + TIME*ChaosSpeed*1.2) + sin(p.z*5.0 + TIME*ChaosSpeed*0.8)) * chaos;
}

float scene(vec3 p, int geo, float chaos, float mixAmt, float fractalControlScale) {
    float base;
    if (geo == 0) {
        base = length(p) - (1.0 + fractalControlScale * 0.3);
    } else if (geo == 1) {
        vec2 q = vec2(length(p.xz) - (1.0 + fractalControlScale * 0.2), p.y);
        base = length(q) - 0.3;
    } else if (geo == 2) {
        base = shapeSpikeFractal(p * (1.0 + fractalControlScale * 0.5), int(8.0 + fractalControlScale * 20.0));
    } else if (geo == 3) {
        // layered spike + sphere
        base = min(length(p) - 1.0, shapeSpikeFractal(p * 1.2, 12));
    } else if (geo == 4) {
        // repeated tiling fractal
        vec3 pp = fract(p * 0.8) - 0.5;
        base = shapeSpikeFractal(pp, 10);
    } else {
        base = shapeSpikeFractal(p, 14);
    }
    return mix(base, shapeChaos(p, chaos), mixAmt);
}

// ------------------------- Transforms (apply every iteration) -------------------------
vec3 applyTransform(vec3 p, int mode, float chaos, float sym, float chspd, float morph, float warping, float fractalControlScale) {
    // symmetry scale
    p *= max(sym, 0.0001);
    if (mode == 0) {
        // none
    } else if (mode == 1) {
        p = abs(p);
    } else if (mode == 2) {
        p += sin(p * (2.0 + warping) + TIME * chspd) * chaos * 0.25;
    } else if (mode == 3) {
        p += sin(p * (1.0 + chaos * 2.0) + TIME * chspd) * chaos * 0.5;
        p = fract(p * (1.2 + fractalControlScale * 0.5)) - 0.6;
    } else if (mode == 4 || mode == 5) {
        float a = atan(p.z, p.x);
        float r = length(p.xz);
        float spin = TIME * chspd * (mode == 4 ? 0.2 : 0.35) * (1.0 + fractalControlScale * 0.25);
        a += spin;
        p.x = cos(a) * r;
        p.z = sin(a) * r;
    }
    // morphing-driven warp
    p += sin(vec3(p.zx, p.yz) * (2.0 + morph * 3.0) + TIME * (0.3 + chspd*0.2)) * (chaos * 0.15);
    // subtle rotation/scaling from warping and fractalControl
    p.xy *= rot2d(TIME * 0.05 * (1.0 + warping));
    p *= 1.0 + fractalControlScale * 0.02;
    return p;
}

// ------------------------- Color helpers -------------------------
vec3 applyContrast(vec3 col, float contrast) {
    return (col - 0.5) * contrast + 0.5;
}
vec3 applySaturation(vec3 col, float sat) {
    float lum = dot(col, vec3(0.2126, 0.7152, 0.0722));
    return mix(vec3(lum), col, sat);
}

// ------------------------- Main rendering -------------------------
void main() {
    // normalized coordinates [-1,1] with aspect correction
    vec2 ndc = isf_FragNormCoord * 2.0 - 1.0;
    vec2 aspect = vec2(RENDERSIZE.x / RENDERSIZE.y, 1.0);
    vec2 uv = ndc * aspect;

    // combine both Speed/ speed controls for layered modulation
    float combinedSpeed = Speed * (1.0 + speed * 0.5);
    float t = TIME * combinedSpeed;

    // combine Zoom controls: Zoom primary, zoom secondary as micro-zoom
    float combinedZoom = Zoom * (1.0 + zoom * 0.2);

    // camera
    vec3 ro = vec3(0.0, 0.0, -3.0);
    vec3 rd = normalize(vec3(uv * combinedZoom * FOV, 1.0));
    rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

    // triplanar displacement for origin (texture influence)
    vec3 warpOrigin = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
    // combine both texture warp sliders and Warping param
    float combinedTextureWarp = TextureWarp * (1.0 + Warping * 0.25);
    vec3 roWarped = ro + warpOrigin * combinedTextureWarp;

    // compute step limit using StepCount and MAX_ITERATIONS/FractalControl (both refs used)
    int stepLimit = int(clamp(StepCount + MAX_ITERATIONS * FractalControl, 1.0, float(MAX_STEPS)));
    // ensure compile-time loop limit:
    if (stepLimit > MAX_STEPS) stepLimit = MAX_STEPS;

    // gather other params
    int transformMode = int(clamp(TransformMode, 0.0, 5.0));
    int geo = int(clamp(GeometryType, 0.0, 6.0));
    float chaos = ChaosIntensity;
    float chaosMix = ChaosMix;
    float sym = Symmetry;
    float chspd = ChaosSpeed;
    float br = Brightness * brightness; // combine both brightness sliders (ref merge)
    float ct = Contrast;
    float glow = Glow;
    int palMode = int(clamp(ColorPaletteMode, 0.0, 19.0));
    float cpPreset = colorPalette;
    float colorPulseAmt = ColorPulse;
    float sharp = Sharpness;
    float falloff = FalloffCurve;
    float morph = Morphing;
    float fractalControlScale = FractalControl;
    float warping = Warping;
    float sat = Saturation;

    // accumulation
    vec3 col = vec3(0.0);
    float dist = 0.0;

    // iterative volumetric raymarch (sphere-tracing-ish but optimized)
    for (int i = 0; i < MAX_STEPS; i++) {
        if (i >= stepLimit) break;

        vec3 p = roWarped + rd * dist;

        // apply iterative transforms (many small transforms combined from both refs)
        p = applyTransform(p, transformMode, chaos, sym, chspd, morph, warping, fractalControlScale);

        // evaluate SDF-like scene
        float d = scene(p, geo, chaos, chaosMix, fractalControlScale);
        // ensure minimal step (avoid zero)
        d = max(abs(d), 0.001);

        // per-step fade (gives depth)
        float fade = exp(-float(i) * 0.03 * sharp);

        // focus / depth of field falloff (combined with FocusNear/Far)
        float focus = smoothstep(FocusNear, FocusFar, dist);
        // compute color from palettes and texture
        float palT = p.z * 0.1 + t * 0.1 + float(i) * 0.001;
        // palette choice: if cpPreset small, use preset palettes, else full palette mode
        vec3 palColA = getColorPalette(palMode, palT * (1.0 + colorPulseAmt * 0.2));
        // allow small legacy colorPalette selection override (second shader)
        vec3 palColB = getColorPalette(int(floor(cpPreset + 0.5)), fract(palT + TIME * 0.1));
        // triplanar sampled texture color inside fractal
        vec3 texCol = triplanarTexture(p * TextureScale * (1.0 + fractalControlScale * 0.2), 1.0);

        // mix palettes with texture — use TextureWarp and Morphing to influence blend
        float texMixFactor = clamp(TextureWarp * 0.5 + morph * 0.25, 0.0, 1.0);
        vec3 mixPal = mix(palColA, palColB, 0.35);
        vec3 finalBase = mix(mixPal, texCol, texMixFactor);

        // contribution weight, cheaper than pow, influenced by distance / density
        float contrib = 0.005 / (0.01 + d * falloff);
        contrib *= fade * focus * (1.0 + glow * 0.15);

        // add tiny noise modulation for organic look
        float noise = (hash21(vec2(float(i), dist)) - 0.5) * 0.015;
        col += finalBase * (contrib + noise);

        // step forward: make steps proportional to estimated distance but clipped
        float stepScale = 0.85 + fractalControlScale * 0.15;
        float step = clamp(d * stepScale, 0.002, 1.5);
        dist += step;

        if (dist > BAILOUT) break;
    }

    // subtle pulsation from combined chaos + colorPulse + speed
    float pulse = sin(TIME * (ChaosSpeed + speed * 0.5 + combinedSpeed * 0.02)) * 0.5 + 0.5;
    col *= 1.0 + 0.3 * pulse * ColorPulse;

    // final color adjustments
    col = applyContrast(col, ct);
    col *= br;
    col = applySaturation(col, sat);

    // optional bloom/feedback: read previous frame from bufferA (if exists) to add persistence
    vec3 feedback = IMG_NORM_PIXEL(bufferA, isf_FragNormCoord).rgb;
    // blend feedback softly influenced by Glow and speed (both refs)
    float feedbackBlend = clamp(0.06 + 0.06 * Glow + 0.02 * speed, 0.0, 0.25);
    // sharpen/soften by mixing with previous buffer — creates trailing and glow
    vec3 outCol = mix(col, feedback, feedbackBlend);

    // write to bufferA (PASSINDEX 0) and final pass will blit buffer
    if (PASSINDEX == 0) {
        // clamp to avoid runaway values; apply slight tone mapping influenced by Glow
        vec3 mapped = outCol / (outCol + vec3(1.0));
        mapped = mix(mapped, outCol, clamp(Glow * 0.5, 0.0, 1.0));
        gl_FragColor = vec4(clamp(mapped, 0.0, 1.0), 1.0);
        return;
    }

    // final pass (PASSINDEX == 1): composite bufferA with a tiny post glare using shifted samples
    if (PASSINDEX == 1) {
        // sample bufferA with a small UV offset to create smear/glow
        vec2 uvOffset = (isf_FragNormCoord - 0.5) * 0.002 * (1.0 + Glow);
        vec3 s1 = IMG_NORM_PIXEL(bufferA, isf_FragNormCoord + uvOffset * 0.6).rgb;
        vec3 s2 = IMG_NORM_PIXEL(bufferA, isf_FragNormCoord - uvOffset * 0.9).rgb;
        vec3 final = mix(IMG_NORM_PIXEL(bufferA, isf_FragNormCoord).rgb, s1 * 0.6 + s2 * 0.4, clamp(Glow * 0.8, 0.0, 1.0));
        // extra contrast + brightness and clamp
        final = applyContrast(final, ct);
        final *= br;
        final = applySaturation(final, sat);
        gl_FragColor = vec4(clamp(final, 0.0, 1.0), 1.0);
        return;
    }

    // fallback safety (shouldn't reach)
    gl_FragColor = vec4(clamp(outCol, 0.0, 1.0), 1.0);
}
