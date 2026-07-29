/*
{
    "CATEGORIES": ["Fractal", "Psychedelic", "Volumetric"],
    "DESCRIPTION": "The original 2D fractal engine is now the core geometry for a 3D volumetric raymarcher, modulated by all the extra features from the reference ISF. This version corrects the GeometryType bug and adds more complex shapes.",
    "ISFVSN": "2.0",
    "INPUTS": [
        { "NAME": "Speed", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.1, "MAX": 50.0, "LABEL": "Animation Speed" },
        { "NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0, "LABEL": "Camera Zoom" },
        { "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5, "LABEL": "Geometric Transform Mode" },
        { "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6, "LABEL": "Geometry Type" },
        { "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0, "LABEL": "Chaos Intensity" },
        { "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0, "LABEL": "Chaos Animation Speed" },
        { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 19, "MIN": 0, "MAX": 19, "LABEL": "Color Palette Mode" },
        { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0, "MAX": 3.0, "LABEL": "Global Brightness" },
        { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 3.0, "LABEL": "Global Contrast" },
        { "NAME": "Glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0, "LABEL": "Global Glow" },
        { "NAME": "Symmetry", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 4.0, "LABEL": "Symmetry Amount" },
        { "NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0, "LABEL": "Chaos Mix Amount" },
        { "NAME": "Sharpness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0, "LABEL": "Iteration Sharpness" },
        { "NAME": "FalloffCurve", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.1, "MAX": 3.0, "LABEL": "Color Falloff" },
        { "NAME": "CameraOrbit", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "LABEL": "Camera Orbit" },
        { "NAME": "CameraPitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.57, "MAX": 1.57, "LABEL": "Camera Pitch" },
        { "NAME": "CameraRoll", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "LABEL": "Camera Roll" },
        { "NAME": "FocusNear", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0, "LABEL": "Focus Near" },
        { "NAME": "FocusFar", "TYPE": "float", "DEFAULT": 2.6, "MIN": 0.1, "MAX": 10.0, "LABEL": "Focus Far" },
        { "NAME": "FOV", "TYPE": "float", "DEFAULT": 1.6, "MIN": 0.2, "MAX": 3.0, "LABEL": "Field of View" },
        { "NAME": "StepCount", "TYPE": "float", "DEFAULT": 60, "MIN": 1, "MAX": 120, "LABEL": "Raymarch Steps" },
        { "NAME": "Texture", "TYPE": "image", "LABEL": "Input Texture" },
        { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0, "LABEL": "Texture Warp Intensity" },
        { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0, "LABEL": "Texture Scale" },
       
        { "NAME": "FractalScale", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.01, "MAX": 0.5, "LABEL": "Original Fractal Scale" },
        { "NAME": "MorphAmount", "TYPE": "float", "DEFAULT": 0.01, "MIN": 0.0, "MAX": 1.0, "LABEL": "Original Morph Amount" },
        { "NAME": "Pulse", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0, "LABEL": "Original Color Pulse Speed" },
        { "NAME": "RedShiftSpeed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0, "LABEL": "Original Red Pulse Speed" },
        { "NAME": "BlueShiftSpeed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0, "LABEL": "Original Blue Pulse Speed" },
        { "NAME": "GeometryDeform", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 2.0, "LABEL": "Original Geometry Deform Scale" },
        { "NAME": "TrapBias", "TYPE": "float", "DEFAULT": 0.15, "MIN": 0.0, "MAX": 1.0, "LABEL": "Original Orbit Trap Bias" }
    ]
}
*/

#define BAILOUT 16.0
#define PI 3.14159
#define TAU 6.28318

// --- Helper Functions ---
#define rot(a) mat2(cos(a), sin(a), -sin(a), cos(a))

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

// --- Color Palette Functions ---
vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
    return a + b * cos(TAU * (c * t + d));
}

vec3 getColorPalette(int mode, float t) {
    return pal(t,
        vec3(0.5 + 0.4*sin(float(mode)*0.5), 0.6 + 0.3*cos(float(mode)*1.2), 0.4 + 0.5*sin(float(mode)*0.9)),
        vec3(0.4),
        vec3(1.0,1.3,0.7),
        vec3(0.1,0.2,0.3)
    );
}

// --- Texture Functions ---
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

// --- Core Original Fractal Logic (Modified for 3D) ---
float originalFractalSDF(vec3 p, float time) {
    // Project the 3D point onto the XY plane and apply original transforms
    vec2 pos = p.xy;
    pos *= rot(time * 0.1) * FractalScale;
    pos.y -= 0.2266;
    pos.x += 0.2082;
    pos.y += p.z * 0.5; // Add some Z-depth to the 2D plane

    vec2 ot = vec2(100.0);
    float m = 100.0;

    for (int i = 0; i < 20; i++) { // Reduced iterations for performance
        vec2 cp = vec2(pos.x, -pos.y);
        float denom = dot(pos, pos);
        pos = pos + cp / denom - vec2(0.0, 0.25 + MorphAmount);
        pos *= 0.1 + MorphAmount * GeometryDeform;
        pos *= rot(1.5 + MorphAmount * 0.5);
        ot = min(ot, abs(pos) + TrapBias * fract(max(abs(pos.x), abs(pos.y)) * 0.25 + time * 0.1 + float(i) * 0.15));
        m = min(m, abs(pos.y));
    }
    
    // The SDF is based on the bailout value 'ot' from the original fractal
    float d = length(ot);

    // Use a mix of the two bailout values for a richer shape
    d = mix(d, m, 0.5) * 0.05;

    return d;
}

// --- Basic SDFs ---
float sdSphere(vec3 p, float s) { return length(p) - s; }
float sdBox(vec3 p, vec3 b) { vec3 q = abs(p) - b; return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0); }
float sdTorus(vec3 p, vec2 t) { vec2 q = vec2(length(p.xz)-t.x,p.y); return length(q)-t.y; }
float sdPlane(vec3 p) { return p.y; }
float sdHexPrism(vec3 p, vec2 h) {
    vec3 q = abs(p);
    return max(q.z-h.y,max(q.x+q.y*0.57735,q.y*1.1547)-h.x);
}

// --- Scene SDFs ---
float shapeChaos(vec3 p, float chaos) {
    return (sin(p.x*3. + TIME*ChaosSpeed) + sin(p.y*4. + TIME*ChaosSpeed*1.2) + sin(p.z*5. + TIME*ChaosSpeed*0.8)) * chaos;
}

float scene(vec3 p, int geo, float chaos, float mixAmt, float time) {
    float base;
    // The core fractal is always the starting point
    float fractal_dist = originalFractalSDF(p, time);

    // Now, modulate the base shape with the fractal, based on GeometryType
    if (geo == 0) {
        base = fractal_dist; // Only the original fractal
    } else if (geo == 1) {
        float primitive_dist = sdSphere(p, 1.0);
        base = mix(primitive_dist, fractal_dist, 0.5);
    } else if (geo == 2) {
        float primitive_dist = sdBox(p, vec3(1.0));
        base = mix(primitive_dist, fractal_dist, 0.5);
    } else if (geo == 3) {
        float primitive_dist = sdTorus(p, vec2(1.0, 0.3));
        base = mix(primitive_dist, fractal_dist, 0.5);
    } else if (geo == 4) {
        float primitive_dist = sdPlane(p);
        base = min(primitive_dist, fractal_dist);
    } else if (geo == 5) {
        float primitive_dist = sdHexPrism(p, vec2(1.0, 1.0));
        base = mix(primitive_dist, fractal_dist, 0.5);
    } else if (geo == 6) {
        // Menger-like fractal by subtracting fractal from box
        float primitive_dist = sdBox(p, vec3(1.0));
        base = max(primitive_dist, -fractal_dist * 2.0);
    } else {
        base = fractal_dist; // Default to the original
    }
    
    // The main chaos deformation is applied after the core shape is determined
    return mix(base, shapeChaos(p, chaos), mixAmt);
}

// --- Transformation Functions ---
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

// --- Main Shader Execution ---
void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
    uv *= FOV;
    float t = TIME * Speed;

    vec3 ro = vec3(0.0, 0.0, -3.0);
    vec3 rd = normalize(vec3(uv * Zoom, 1.0));
    rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;

    vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
    vec3 roWarped = ro + warp * TextureWarp;

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
    
    // Original fractal color variables
    float pulseSpeed = Pulse;
    float redShift = RedShiftSpeed;
    float blueShift = BlueShiftSpeed;
    
    for (int i = 0; i < int(StepCount); i++) {
        vec3 p = roWarped + dist * rd;
        p = applyTransform(p, mode, chaos, sym, chspd);
        float d = scene(p, geo, chaos, chaosMix, t);
        d = max(abs(d), 0.01);

        float fade = exp(-float(i)*0.03*sharp);
        float focus = smoothstep(FocusNear, FocusFar, dist);

        // Calculate original fractal color to modulate the new palettes
        float pulse = sin(t * pulseSpeed + length(p)) * 0.5 + 0.5;
        vec3 originalPalColor = vec3(0.6 + 0.4*sin(TAU * pulse + vec3(0.1, 0.3, 0.5)));
        originalPalColor.r *= sin(t * redShift + length(p)) * 0.5 + 0.5;
        originalPalColor.b *= cos(t * blueShift + length(p)) * 0.5 + 0.5;

        // New volumetric palette
        vec3 volumetricPalCol = getColorPalette(pal, p.z + t * 0.1);

        // Mix the original fractal color with the new volumetric palette and texture
        vec3 finalCol = mix(originalPalColor, volumetricPalCol, 0.5);
        vec3 texCol = triplanarTexture(p * TextureScale, 1.0);
        finalCol = mix(finalCol, texCol, 0.5);

        float b = 0.005 / (0.01 + d * falloff);
        col += finalCol * b * fade * focus;

        dist += d;
        if (dist > BAILOUT) break;
    }

    col = (col - 0.5) * ct + 0.5;
    col *= br;
    col *= (1.0 + glow * (1.0 - col)) * col;

    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
