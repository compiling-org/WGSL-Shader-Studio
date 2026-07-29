/*{
    "DESCRIPTION": "3D Tunnel Fractal × Liminal Chaos — merged raymarcher: path-following tunnel + mixed SDF/fractal geometry, palettes, triplanar, and camera controls.",
    "CATEGORIES": ["Generator","Fractal","Volumetric","Psychedelic","Raymarching","3D","Tunnel"],
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
        { "NAME": "StepCount", "TYPE": "float", "DEFAULT": 32.0, "MIN": 1.0, "MAX": 128.0 },
        { "NAME": "Texture", "TYPE": "image" },
        { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0 },
        { "NAME": "colorPulse", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "shakeAmount", "TYPE": "float", "DEFAULT": 0.01, "MIN": 0.0, "MAX": 0.1 },
        { "NAME": "cameraShift", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
        { "NAME": "fractalMorph", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "LiminalMix", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "LiminalScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0 },

        { "NAME": "P_X_CosFactor", "TYPE": "float", "DEFAULT": 0.30, "MIN": 0.05, "MAX": 1.5 },
        { "NAME": "P_X_Amp",       "TYPE": "float", "DEFAULT": 0.50, "MIN": 0.0,  "MAX": 3.0 },
        { "NAME": "P_X_Range",     "TYPE": "float", "DEFAULT": 6.00, "MIN": 0.0,  "MAX": 30.0 },
        { "NAME": "P_Y_CosFactor", "TYPE": "float", "DEFAULT": 0.40, "MIN": 0.05, "MAX": 1.5 },
        { "NAME": "P_Y_Amp",       "TYPE": "float", "DEFAULT": 0.30, "MIN": 0.0,  "MAX": 3.0 },
        { "NAME": "P_Y_Range",     "TYPE": "float", "DEFAULT": 16.0, "MIN": 0.0,  "MAX": 40.0 },
        { "NAME": "TunnelRadius",  "TYPE": "float", "DEFAULT": 1.05, "MIN": 0.2,  "MAX": 4.0 },
        { "NAME": "TunnelSoft",    "TYPE": "float", "DEFAULT": 0.20, "MIN": 0.01, "MAX": 1.0 }
    ]
}*/

#define PI 3.14159265359
#define TAU 6.28318530718
#define BAILOUT 24.0

mat2 rot(float a){ float s=sin(a), c=cos(a); return mat2(c, s, -s, c); }
mat3 cameraMatrix(float orbit, float pitch, float roll){
    float co=cos(orbit), so=sin(orbit);
    float cp=cos(pitch), sp=sin(pitch);
    float cr=cos(roll), sr=sin(roll);
    return mat3(
        co*cr + so*sp*sr,  sr*cp, -so*cr + co*sp*sr,
       -co*sr + so*sp*cr,  cr*cp,  sr*so + co*sp*cr,
         so*cp,           -sp,     co*cp
    );
}

/* ---------- Path function (tunnel centerline) ---------- */
vec3 P(float z){
    return vec3(
        tanh(cos(z * P_X_CosFactor) * P_X_Amp) * P_X_Range,
        tanh(cos(z * P_Y_CosFactor) * P_Y_Amp) * P_Y_Range,
        z
    );
}

/* ---------- Palettes ---------- */
vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d){ return a + b * cos(TAU*(c*t + d)); }

vec3 getLiminalPalette(float t, float id){
    if (id < 1.0) return 0.5 + 0.5 * cos(TAU * (t + vec3(0.0, 0.33, 0.67)));
    if (id < 2.0) return vec3(sin(t*3.0), sin(t*2.5+1.0), sin(t*4.0+2.0)) * 1.1;
    if (id < 3.0) return vec3(1.0 - abs(sin(t*3.0 + vec3(0.5,0.3,0.1)))) * 1.2;
    if (id < 4.0) return vec3(0.3 + 0.4 * sin(t + vec3(1,2,3)));
    if (id < 5.0) return vec3(sin(t*7.0), sin(t*13.0), sin(t*17.0));
    if (id < 6.0) return vec3(1.0, 0.7 + 0.3*sin(t*3.5), 0.6*sin(t*2.0));
    if (id < 7.0) return vec3(exp(-t*2.0)) * vec3(1.2, 0.8, 1.5);
    return 0.5 + 0.5 * cos(TAU * t + vec3(0.0, 0.6, 1.2));
}

vec3 getColorPalette(float mode, float t){
    if (mode < 8.0) return getLiminalPalette(t, mode);
    return pal(t,
        vec3(0.5 + 0.4*sin(float(mode)*0.5), 0.6 + 0.3*cos(float(mode)*1.2), 0.4 + 0.5*sin(float(mode)*0.9)),
        vec3(0.4),
        vec3(1.0,1.3,0.7),
        vec3(0.1,0.2,0.3)
    );
}

/* ---------- Triplanar ---------- */
vec3 triplanarTexture(vec3 p, float scale){
    vec3 n = normalize(abs(p) + 1e-6);
    vec3 blend = pow(n, vec3(4.0)); blend /= dot(blend, vec3(1.0));
    vec2 xz = fract(p.zy * scale);
    vec2 yz = fract(p.xz * scale);
    vec2 xy = fract(p.xy * scale);
    vec3 tx = IMG_NORM_PIXEL(Texture, xz).rgb;
    vec3 ty = IMG_NORM_PIXEL(Texture, yz).rgb;
    vec3 tz = IMG_NORM_PIXEL(Texture, xy).rgb;
    return tx * blend.x + ty * blend.y + tz * blend.z;
}

/* ---------- SDF / fractal pieces ---------- */
float sdSphere(vec3 p, float r){ return length(p) - r; }
float sdTorus(vec3 p, vec2 t){ vec2 q = vec2(length(p.xz)-t.x, p.y); return length(q) - t.y; }
float sdGrid(vec3 p, float f){ return sin(p.x*f) * sin(p.y*f) * sin(p.z*f); }

float shapeSpikeFractal(vec3 p){
    float d=0.0;
    int K = int(clamp(StepCount, 1.0, 128.0));
    for(int i=0;i<128;i++){
        if(i>=K) break;
        p = abs(p) / (dot(p,p) + 0.001) - 0.5;
        p *= 0.95;
        d += length(p);
    }
    return d/20.0;
}

float liminalFractal(vec3 p, float t, float scale){
    p *= scale;
    float m = 1e9;
    for(int i=0;i<7;i++){
        p = abs(p) / max(dot(p,p), 1e-4) - fractalMorph;
        p.xy *= rot(t*0.05 + float(i)*0.1);
        m = min(m, length(p));
    }
    return m - 0.6; /* shift for usability */
}

float shapeChaos(vec3 p, float chaos){
    return (sin(p.x*3. + TIME*ChaosSpeed)
          + sin(p.y*4. + TIME*ChaosSpeed*1.2)
          + sin(p.z*5. + TIME*ChaosSpeed*0.8)) * chaos;
}

/* ---------- 2D liminal warp (applied in 3D plane) ---------- */
vec3 applyLiminalWarp(vec3 p, float t){
    p.xy *= Zoom + dot(p.xy,p.xy)*2.0;
    float s = sin(t*0.1*1.585*PI);
    p.x += s*s*s;
    p.xy *= rot(tan(t*PI*1.585*.2)*.1);
    p.xy *= 7.0 - atan(5.0 * cos(t*TAU*1.585*.25) * PI * 1.585) * 1.5;
    return p;
}

/* ---------- Transform stack ---------- */
vec3 applyTransform(vec3 p, float mode, float chaos, float sym, float chspd){
    p *= max(sym, 0.001);
    if (mode < 1.5){
        p = abs(p);
    } else if (mode < 2.5){
        p += sin(p*3.0 + TIME*chspd) * chaos * 0.3;
    } else if (mode < 3.5){
        p += sin(p * (1.0 + chaos*2.0) + TIME*chspd) * chaos * 0.5;
        p = fract(p*1.5) - 0.75;
    }
    if (mode > 3.5 && mode < 5.5){
        float a = atan(p.z, p.x);
        float r = max(1e-4, length(p.xz));
        float spin = TIME * chspd * (mode < 4.5 ? 0.2 : 0.3);
        a += spin;
        p.x = cos(a)*r;
        p.z = sin(a)*r;
    }
    return p;
}

/* ---------- Scene: mix tunnel SDF with fractal choices ---------- */
float sceneCore(vec3 p, float t, float geo, float chaos, float mixAmt, float limMix, float limScale){
    float base;
    if (geo < 0.5) base = sdSphere(p, 1.0);
    else if (geo < 1.5) base = sdTorus(p, vec2(1.0, 0.3));
    else if (geo < 2.5) base = shapeSpikeFractal(p);
    else if (geo < 3.5) base = sdGrid(p, 4.0);
    else if (geo < 4.5) base = liminalFractal(p, t, limScale);
    else if (geo < 5.5) base = mix(liminalFractal(p, t, limScale), sdGrid(p, 4.0)*0.1, 0.5);
    else base = mix(shapeSpikeFractal(p), sdSphere(p, 1.0), 0.5);

    float lim = liminalFractal(p, t, limScale);
    float spike = shapeSpikeFractal(p);
    float finalShape = mix(spike, lim, limMix);

    float chaotic = shapeChaos(p, chaos);
    float mixed = mix(finalShape, chaotic, mixAmt);

    /* Soften */
    return mix(base, mixed, 0.6);
}

/* Signed distance to tunnel hull around path P */
float sdTunnel(vec3 p, float t){
    /* subtract path centerline */
    vec3 c = P(p.z + t*0.3);
    vec2 d = p.xy - c.xy;
    return length(d) - TunnelRadius;
}

/* Combine tunnel and core scene (union with smooth min) */
float smin(float a, float b, float k){
    float h = clamp(0.5 + 0.5*(b-a)/k, 0.0, 1.0);
    return mix(b, a, h) - k*h*(1.0 - h);
}
float sceneAll(vec3 p, float t, float geo, float chaos, float mixAmt, float limMix, float limScale){
    /* Liminal warp blend */
    vec3 pw = mix(p, applyLiminalWarp(p, t), limMix);

    /* Apply transform stack */
    pw = applyTransform(pw, TransformMode, chaos, Symmetry, ChaosSpeed);

    /* Tunnel coordinates: offset by path center for “coaxial” detail */
    vec3 q = pw;
    q.xy -= P(q.z + t*0.4).xy;

    float core = sceneCore(q, t, geo, chaos, mixAmt, limMix, limScale);
    float tube = sdTunnel(pw, t);

    /* Smoothly union tunnel shell with core structures, keeping a soft wall thickness */
    float k = TunnelSoft;
    float d = smin(core, tube, k);

    /* Keep scene signed-ish; clamp to small positive to avoid stuck rays */
    return d;
}

void main(){
    vec2 uv = (gl_FragCoord.xy - 0.5*RENDERSIZE.xy) / RENDERSIZE.y;
    uv += sin(vec2(TIME*13.1, TIME*11.5)) * shakeAmount;
    uv.x += cameraShift;
    uv *= FOV;

    float t = TIME * Speed;

    /* Camera */
    vec3 ro = vec3(0.0, 0.0, -3.0);
    /* Ride the path slightly to feel motion */
    ro.xy += P(t*0.6).xy * 0.2;

    vec3 rd = normalize(vec3(uv * Zoom, 1.0));
    rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

    /* Texture warp of origin */
    vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
    vec3 roWarped = ro + warp * TextureWarp;

    /* Params cache */
    float geo   = GeometryType;
    float chaos = ChaosIntensity;
    float mixAmt= ChaosMix;
    float limMix= LiminalMix;
    float limSc = LiminalScale;
    float sharp = Sharpness;
    float fall  = FalloffCurve;
    float br    = Brightness;
    float ct    = Contrast;
    float glow  = Glow;
    float palM  = ColorPaletteMode;

    vec3 col = vec3(0.0);
    float dist = 0.0;

    int MAX_STEPS = int(clamp(StepCount, 1.0, 128.0));

    for (int i = 0; i < 256; i++){
        if (i >= MAX_STEPS) break;

        vec3 p = roWarped + dist * rd;

        /* Scene distance */
        float d = sceneAll(p, t, geo, chaos, mixAmt, limMix, limSc);

        /* Safety: keep progress */
        float ad = max(abs(d), 0.005);

        /* Shading weights */
        float fade   = exp(-float(i) * 0.03 * sharp);
        float focus  = smoothstep(FocusNear, FocusFar, dist);

        /* Pal & texture */
        vec3 palCol = getColorPalette(palM, p.z*0.25 + t*0.1) * colorPulse;
        vec3 texCol = triplanarTexture(p * TextureScale, 1.0);

        /* Volumetric-ish accumulation (inverse-square-ish) */
        float w = 0.006 / (0.01 + ad * fall);
        vec3 sampleCol = mix(palCol, texCol, 0.5);

        /* Add subtle tunnel rim lighting by sampling gradient-ish cue */
        float tube = sdTunnel(p, t);
        float rim  = smoothstep(0.2, 0.0, abs(tube));
        sampleCol *= mix(1.0, 1.0 + rim*0.75, 0.8);

        col += sampleCol * w * fade * focus;

        dist += ad;
        if (dist > BAILOUT) break;
    }

    /* Glow & tone */
    float pulse = sin(TIME * ChaosSpeed) * 0.5 + 0.5;
    col *= (1.0 + glow*0.5) * (1.0 + 0.25 * pulse);

    /* Contrast around 0.5, then brightness */
    col = (col - 0.5) * ct + 0.5;
    col *= br;

    /* Gentle gamma */
    col = pow(max(col, 0.0), vec3(0.9));

    /* Vignette */
    vec2 nuv = (gl_FragCoord.xy/RENDERSIZE.xy) - 0.5;
    float vig = 1.0 - dot(nuv, nuv) * 1.5;
    col *= clamp(vig, 0.0, 1.0);

    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
