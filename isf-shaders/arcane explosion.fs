/*{
  "DESCRIPTION": "Particle shader upgraded: fully merged with the reference fractal ISF features, volumetric raymarch, triplanar texture influence, full palette modes, camera controls, chaos/morph transforms, persistence buffer, many sliders merged from both references.",
  "CATEGORIES": ["Psychedelic","Particles","Volumetric","Fractal"],
  "ISFVSN": "2.0",
  "INPUTS": [
    { "NAME": "ParticleSpinSpeed", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 2.0, "LABEL": "Particle Spin Speed" },
    { "NAME": "ShapeWarpAmount", "TYPE": "float", "DEFAULT": 2.8, "MIN": 0.1, "MAX": 5.0, "LABEL": "Shape Warp Amount" },
    { "NAME": "GlowIntensity", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 3.0, "LABEL": "Glow Intensity" },
    { "NAME": "ColorRotationSpeed", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.0, "MAX": 2.0, "LABEL": "Color Spin Speed" },
    { "NAME": "ColorScheme", "TYPE": "float", "DEFAULT": 0, "MIN": 0, "MAX": 3, "LABEL": "Color Scheme", "VALUES": ["Firestorm","Aurora","Plasma","Rainbow"] },

    
    { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0 },
    { "NAME": "speed", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.01, "MAX": 2.0 },

    { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0 },
    { "NAME": "zoom", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.1, "MAX": 5.0 },

    { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5 },
    { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6 },

    { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0 },

    { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 19, "MIN": 0, "MAX": 19 },
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
    { "TARGET": "bufferA", "PERSISTENT": true }
  ]
}*/

#define PI 3.14159265359
#define MAX_STEPS 128
#define BAILOUT 16.0

// ------------------------- small utilities -------------------------
mat2 rot(float a) { float s = sin(a), c = cos(a); return mat2(c, s, -s, c); }
mat2 rot2d(float a) { return mat2(cos(a), -sin(a), sin(a), cos(a)); }

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

float hash21(vec2 p) { p = fract(p * vec2(123.34,456.21)); p += dot(p, p+78.233); return fract(p.x * p.y); }
float hash(vec2 p) { vec3 p3 = fract(vec3(p.xyx) * 0.1031); p3 += dot(p3, p3.yzx + 33.33); return fract((p3.x + p3.y) * p3.z); }

// ------------------------- palettes -------------------------
vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
    return a + b * cos(6.28318530718 * (c * t + d));
}
vec3 paletteRainbow(float t) { return vec3(0.5 + 0.5 * cos(6.28318 * (t + vec3(0.0,0.33,0.67)))); }
vec3 paletteNeon(float t)   { return vec3(0.5 + 0.5 * sin(6.28318 * (t + vec3(0.0,0.2,0.4)))); }
vec3 paletteFire(float t)   { return vec3(pow(t,0.6), pow(t,0.3), 0.0); }
vec3 paletteIce(float t)    { return vec3(0.0, pow(t,0.5)*0.8, pow(t,0.3)*1.2); }

vec3 getColorPalette(int mode, float t) {
    if(mode == 0) return paletteRainbow(t);
    if(mode == 1) return paletteNeon(t);
    if(mode == 2) return paletteFire(t);
    if(mode == 3) return paletteIce(t);
    float m = float(mode);
    vec3 a = vec3(0.5 + 0.4*sin(m*0.5 + 0.1), 0.6 + 0.3*cos(m*1.2 + 0.2), 0.4 + 0.5*sin(m*0.9 - 0.3));
    vec3 b = vec3(0.5, 0.4, 0.35) * (1.0 + 0.02 * m);
    vec3 c = vec3(1.0, 1.3, 0.7) + 0.02 * m;
    vec3 d = vec3(0.1, 0.2, 0.3) + 0.01 * m;
    return pal(t, a, b, c, d);
}

// ------------------------- triplanar texture -------------------------
vec3 triplanarTexture(vec3 p, float scale) {
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

// ------------------------- fractal/particle distance evaluations -------------------------
float shapeSpikeFractal(vec3 p, int iters) {
    float d = 0.0;
    for (int i = 0; i < 128; i++) {
        if (i >= iters) break;
        float denom = dot(p,p) + 0.001;
        p = abs(p) / denom - 0.5;
        p *= 0.95;
        d += length(p);
    }
    return d / 20.0;
}

float shapeChaos(vec3 p, float chaos) {
    return (sin(p.x*3.0 + TIME*ChaosSpeed) + sin(p.y*4.0 + TIME*ChaosSpeed*1.2) + sin(p.z*5.0 + TIME*ChaosSpeed*0.8)) * chaos;
}

float sceneSDF(vec3 p, int geo, float chaos, float mixAmt, float fractalControlScale) {
    float base;
    if (geo == 0) {
        base = length(p) - (1.0 + fractalControlScale * 0.3);
    } else if (geo == 1) {
        vec2 q = vec2(length(p.xz) - (1.0 + fractalControlScale * 0.2), p.y);
        base = length(q) - 0.3;
    } else if (geo == 2) {
        base = shapeSpikeFractal(p * (1.0 + fractalControlScale * 0.5), int(8.0 + fractalControlScale * 20.0));
    } else if (geo == 3) {
        base = min(length(p) - 1.0, shapeSpikeFractal(p * 1.2, 12));
    } else if (geo == 4) {
        vec3 pp = fract(p * 0.8) - 0.5;
        base = shapeSpikeFractal(pp, 10);
    } else {
        base = shapeSpikeFractal(p, 14);
    }
    return mix(base, shapeChaos(p, chaos), mixAmt);
}

// ------------------------- transform applied to sample points -------------------------
vec3 applyTransform(vec3 p, int mode, float chaos, float sym, float chspd, float morph, float warping, float fractalControlScale) {
    p *= max(sym, 0.0001);
    if (mode == 1) p = abs(p);
    else if (mode == 2) p += sin(p * (2.0 + warping) + TIME * chspd) * chaos * 0.25;
    else if (mode == 3) {
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
    p += sin(vec3(p.zx, p.yz) * (2.0 + morph * 3.0) + TIME * (0.3 + chspd*0.2)) * (chaos * 0.15);
    p.xy *= rot2d(TIME * 0.05 * (1.0 + warping));
    p *= 1.0 + fractalControlScale * 0.02;
    return p;
}

// ------------------------- color tweaks -------------------------
vec3 applyContrast(vec3 col, float contrast) { return (col - 0.5) * contrast + 0.5; }
vec3 applySaturation(vec3 col, float sat) {
    float lum = dot(col, vec3(0.2126,0.7152,0.0722));
    return mix(vec3(lum), col, sat);
}

// ------------------------- original particle DE adapted and merged -------------------------
float particleDE(vec3 p, float ParticleSpinSpeed_local, float ShapeWarpAmount_local) {
    // Combining original iterative deformation logic but constrained
    p.yz *= rot(-0.5);
    p.xz *= rot(TIME * ParticleSpinSpeed_local);
    float d = 100.0;
    p *= 0.2;
    for (int i = 0; i < 12; i++) {
        p.xy = sin(p.xy * ShapeWarpAmount_local);
        p.xy *= rot(1.0);
        p.xz *= rot(1.5);
        float l = length(p.xy) + 0.01;
        if (i > 1) d = min(d, l);
    }
    return d * 0.3;
}

// ------------------------- volumetric marcher for particles (core) -------------------------
vec3 marchParticles(vec3 from, vec3 dir,
                    float ParticleSpinSpeed_local, float ShapeWarpAmount_local,
                    int transformMode, int geo, float chaos, float chaosMix,
                    float sym, float chspd, float morph, float warping, float fractalControlScale,
                    int palMode, float cpPreset, float ColorRotationSpeed_local, int colorSchemeChoice,
                    float TextureWarp_local, float TextureScale_local,
                    float Brightness_local, float Saturation_local, float Glow_local,
                    float Sharpness_local, float FalloffCurve_local) {

    // triplanar warp sampled at origin, used to perturb rays lightly
    vec3 col = vec3(0.0);
    float td = hash(gl_FragCoord.xy + TIME) * 0.2; // initial t
    vec3 p;
    float dist = 0.0;
    int stepLimit = int(clamp(StepCount + MAX_ITERATIONS * FractalControl, 1.0, float(MAX_STEPS)));
    if (stepLimit > MAX_STEPS) stepLimit = MAX_STEPS;

    // small origin warp
    vec3 warpOrigin = triplanarTexture(from * TextureScale_local, 1.0) - 0.5;
    from += warpOrigin * TextureWarp_local * 0.5;

    for (int i = 0; i < MAX_STEPS; i++) {
        if (i >= stepLimit) break;

        p = from + dir * dist;

        // apply both particle deformation and fractal transforms
        // run a cheap particle DE to add local variation
        float pd = particleDE(p, ParticleSpinSpeed_local, ShapeWarpAmount_local) * 0.5;
        // fractal scene SDF
        vec3 ptx = applyTransform(p, transformMode, chaos, sym, chspd, morph, warping, fractalControlScale);
        float d = sceneSDF(ptx, geo, chaos, chaosMix, fractalControlScale);
        // blend with particle-derived distance to get hybrid shape
        d = mix(d, pd, 0.35);

        // clamp minimal step to avoid stalls
        d = max(abs(d), 0.001);

        // depth fade and focus
        float fade = exp(-float(i) * 0.03 * Sharpness_local);
        float focus = smoothstep(FocusNear, FocusFar, dist);

        // palette selection and texture influence
        float palT = p.z * 0.1 + TIME * (Speed * 0.01 + ColorRotationSpeed_local * 0.1) + float(i) * 0.001;
        vec3 palA = getColorPalette(palMode, palT * (1.0 + ColorPulse * 0.2));
        vec3 palB = getColorPalette(int(floor(cpPreset + 0.5)), fract(palT + TIME * 0.1));
        vec3 texCol = triplanarTexture(ptx * TextureScale_local * (1.0 + fractalControlScale * 0.2), 1.0);

        float texMix = clamp(TextureWarp_local * 0.5 + morph * 0.25, 0.0, 1.0);
        vec3 mixPal = mix(palA, palB, 0.35);
        vec3 finalBase = mix(mixPal, texCol, texMix);

        // original particle color manip from the old shader: build a base c and rotate in color
        vec3 cOrig = vec3(1.0, -0.5, 0.0);
        // rotate channels using a slow rotation influenced by it-like factor (approx)
        float itApprox = float(i) * 0.05;
        cOrig.rg = cOrig.rg * mat2(cos(-itApprox*0.15 + TIME*ColorRotationSpeed_local), sin(-itApprox*0.15 + TIME*ColorRotationSpeed_local), -sin(-itApprox*0.15 + TIME*ColorRotationSpeed_local), cos(-itApprox*0.15 + TIME*ColorRotationSpeed_local));
        cOrig = normalize(1.0 + cOrig);
        // color scheme transform (legacy)
        vec3 cScheme;
        int cs = colorSchemeChoice;
        float tcol = TIME * ColorRotationSpeed_local + dist * 0.1;
        if (cs == 1) cScheme = cOrig.zyx * vec3(sin(tcol), cos(tcol), sin(tcol*0.5));
        else if (cs == 2) cScheme = cOrig * vec3(sin(tcol*0.3), sin(tcol), cos(tcol));
        else if (cs == 3) cScheme = vec3(sin(tcol + cOrig.x), sin(tcol + cOrig.y), sin(tcol + cOrig.z));
        else cScheme = cOrig;

        // blend scheme into finalBase
        finalBase = mix(finalBase, cScheme, 0.35);

        // contribution weight: cheaper than pow
        float contrib = 0.005 / (0.01 + d * FalloffCurve_local);
        contrib *= fade * focus * (1.0 + Glow_local * 0.15);

        // organic noise
        float noise = (hash21(vec2(float(i), dist)) - 0.5) * 0.015;
        col += finalBase * (contrib + noise);

        // also add the original exponential falloffs used in original particle march
        vec3 origCol = vec3(1.0, -0.5, 0.0);
        origCol.rb *= rot(-itApprox * 0.15 + TIME * ColorRotationSpeed_local);
        origCol = normalize(1.0 + origCol);
        vec3 origSchemeCol;
        if (colorSchemeChoice == 1) origSchemeCol = origCol.zyx * vec3(sin(TIME*ColorRotationSpeed_local), cos(TIME*ColorRotationSpeed_local), sin(TIME*ColorRotationSpeed_local*0.5));
        else if (colorSchemeChoice == 2) origSchemeCol = origCol * vec3(sin(TIME*ColorRotationSpeed_local*0.3), sin(TIME*ColorRotationSpeed_local), cos(TIME*ColorRotationSpeed_local));
        else if (colorSchemeChoice == 3) origSchemeCol = vec3(sin(TIME + origCol.x), sin(TIME + origCol.y), sin(TIME + origCol.z));
        else origSchemeCol = origCol;
        origSchemeCol *= exp(-0.15 * dist);
        origSchemeCol *= exp(-0.5 * length(p));
        origSchemeCol /= 1.0 + d * 1500.0;
        origSchemeCol *= (0.3 + abs(pow(abs(fract(length(p) * 0.15 - TIME * 0.2 + itApprox * 0.05) - 0.5) * 2.0, 30.0))) * 4.0;

        col += origSchemeCol * GlowIntensity;
        col += exp(-5.0 * length(p)) * 0.15;

        // step forward using SDF-like stepping
        float stepScale = 0.85 + fractalControlScale * 0.15;
        float step = clamp(d * stepScale, 0.002, 1.5);
        dist += step;

        if (dist > BAILOUT) break;
    }

    // color pulse + modulation
    float pulse = sin(TIME * (ChaosSpeed + speed * 0.5 + Speed * 0.02)) * 0.5 + 0.5;
    col *= 1.0 + 0.3 * pulse * ColorPulse;

    // final adjustments
    col = applyContrast(col, Contrast);
    col *= (Brightness * brightness);
    col = applySaturation(col, Saturation);

    return col;
}

// ------------------------- main -------------------------
void main() {
    // normalized coords
    vec2 ndc = isf_FragNormCoord * 2.0 - 1.0;
    vec2 aspect = vec2(RENDERSIZE.x / RENDERSIZE.y, 1.0);
    vec2 uv = ndc * aspect;

    // camera origin and ray dir (use FOV & Zoom composed)
    float combinedZoom = Zoom * (1.0 + zoom * 0.2);
    vec3 ro = vec3(0.0, 0.0, -3.0);
    vec3 rd = normalize(vec3(uv * combinedZoom * FOV, 1.0));
    rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

    // gather params (local copies to avoid name collisions)
    float ParticleSpinSpeed_local = ParticleSpinSpeed;
    float ShapeWarpAmount_local = ShapeWarpAmount;
    float GlowIntensity_local = GlowIntensity;
    float ColorRotationSpeed_local = ColorRotationSpeed;
    int colorSchemeChoice = int(clamp(ColorScheme, 0.0, 3.0));

    int transformMode = int(clamp(TransformMode, 0.0, 5.0));
    int geo = int(clamp(GeometryType, 0.0, 6.0));
    float chaos = ChaosIntensity;
    float chaosMix = ChaosMix;
    float sym = Symmetry;
    float chspd = ChaosSpeed;
    float morph = Morphing;
    float fractalControlScale = FractalControl;
    float warping = Warping;
    int palMode = int(clamp(ColorPaletteMode, 0.0, 19.0));
    float cpPreset = 0.0; // legacy small palette index (no separate UI here)
    float TextureWarp_local = TextureWarp;
    float TextureScale_local = TextureScale;
    float Brightness_local = Brightness;
    float Saturation_local = Saturation;
    float Glow_local = Glow;
    float Sharpness_local = Sharpness;
    float FalloffCurve_local = FalloffCurve;

    // origin warp from texture
    vec3 warpOrigin = triplanarTexture(ro * TextureScale_local, 1.0) - 0.5;
    ro += warpOrigin * TextureWarp_local * 0.5;

    // compute final color by volumetric marching
    vec3 col = marchParticles(ro, rd,
                             ParticleSpinSpeed_local, ShapeWarpAmount_local,
                             transformMode, geo, chaos, chaosMix,
                             sym, chspd, morph, warping, fractalControlScale,
                             palMode, cpPreset, ColorRotationSpeed_local, colorSchemeChoice,
                             TextureWarp_local, TextureScale_local,
                             Brightness_local, Saturation_local, Glow_local,
                             Sharpness_local, FalloffCurve_local);

    // feedback / persistence using bufferA
    vec3 feedback = IMG_NORM_PIXEL(bufferA, isf_FragNormCoord).rgb;
    float feedbackBlend = clamp(0.06 + 0.06 * Glow_local + 0.02 * speed, 0.0, 0.25);
    vec3 outCol = mix(col, feedback, feedbackBlend);

    // tone mapping and final composite
    vec3 mapped = outCol / (outCol + vec3(1.0));
    mapped = mix(mapped, outCol, clamp(Glow * 0.5, 0.0, 1.0));
    mapped = clamp(mapped, 0.0, 1.0);

    if (PASSINDEX == 0) {
        gl_FragColor = vec4(mapped, 1.0);
        return;
    }

    // final pass: subtle smear for glow if ever used
    if (PASSINDEX == 1) {
        vec2 uvOffset = (isf_FragNormCoord - 0.5) * 0.002 * (1.0 + Glow);
        vec3 s1 = IMG_NORM_PIXEL(bufferA, isf_FragNormCoord + uvOffset * 0.6).rgb;
        vec3 s2 = IMG_NORM_PIXEL(bufferA, isf_FragNormCoord - uvOffset * 0.9).rgb;
        vec3 final = mix(IMG_NORM_PIXEL(bufferA, isf_FragNormCoord).rgb, s1 * 0.6 + s2 * 0.4, clamp(Glow * 0.8, 0.0, 1.0));
        final = applyContrast(final, Contrast);
        final *= (Brightness * brightness);
        final = applySaturation(final, Saturation);
        gl_FragColor = vec4(clamp(final, 0.0, 1.0), 1.0);
        return;
    }

    // fallback
    gl_FragColor = vec4(mapped, 1.0);
}
