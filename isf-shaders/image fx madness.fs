/*
{
    "CATEGORIES": ["Fractal", "Psychedelic", "Abstract", "Texture"],
    "DESCRIPTION": "Enhanced orbit trap fractal with texture warping, robust post-processing, and full control over color, pulse, morph, and geometry, incorporating ideas from a volumetric shader.",
    "ISFVSN": "2",
    "INPUTS": [
        { "NAME": "Speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0, "LABEL": "Animation Speed" },
        { "NAME": "Pulse", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0, "LABEL": "Color Pulse Speed" },
        { "NAME": "FractalScale", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.01, "MAX": 0.5, "LABEL": "Fractal Scale" },
        { "NAME": "MorphAmount", "TYPE": "float", "DEFAULT": 0.01, "MIN": 0.0, "MAX": 1.0, "LABEL": "Morph Amount" },
        { "NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 19, "MIN": 0.0, "MAX": 19.0, "LABEL": "Color Palette Mode" },
        { "NAME": "RedShiftSpeed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0, "LABEL": "Red Pulse Speed" },
        { "NAME": "BlueShiftSpeed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0, "LABEL": "Blue Pulse Speed" },
        { "NAME": "GeometryDeform", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 2.0, "LABEL": "Geometry Deform Scale" },
        { "NAME": "TrapBias", "TYPE": "float", "DEFAULT": 0.15, "MIN": 0.0, "MAX": 1.0, "LABEL": "Orbit Trap Bias" },
        { "NAME": "Texture", "TYPE": "image" },
        { "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0, "LABEL": "Texture Warp Intensity" },
        { "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0, "LABEL": "Texture Scale" },
        { "NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.0, "MAX": 3.0, "LABEL": "Global Brightness" },
        { "NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.1, "MAX": 3.0, "LABEL": "Global Contrast" },
        { "NAME": "Glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0, "LABEL": "Global Glow" },
        { "NAME": "TextureMix", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0, "LABEL": "Texture Mix Amount" }
    ]
}
*/

#define rot(a) mat2(cos(a), sin(a), -sin(a), cos(a))
#define TAU 6.28318

// --- Palette Functions (from previous shader) ---

vec3 pal(float t_val, vec3 a, vec3 b, vec3 c, vec3 d) {
    return a + b * cos(TAU * (c * t_val + d));
}

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

// --- Core Fractal Render Function ---
vec3 render(vec2 p, float time, float scale, float morph, float pulseSpeed, int mode, float redShift, float blueShift, float deform, float trapBias) {
    p *= rot(time * 0.1) * scale;
    p.y -= 0.2266;
    p.x += 0.2082;

    vec2 ot = vec2(100.0);
    float m = 100.0;

    for (int i = 0; i < 150; i++) {
        vec2 cp = vec2(p.x, -p.y);
        float denom = dot(p, p);
        p = p + cp / denom - vec2(0.0, 0.25 + morph);
        p *= 0.1 + morph * deform;
        p *= rot(1.5 + morph * 0.5);
        ot = min(ot, abs(p) + trapBias * fract(max(abs(p.x), abs(p.y)) * 0.25 + time * 0.1 + float(i) * 0.15));
        m = min(m, abs(p.y));
    }

    ot = exp(-200.0 * ot) * 2.0;
    m = exp(-200.0 * m);

    float pulse = sin(time * pulseSpeed + length(p)) * 0.5 + 0.5;
    vec3 palColor = getColorPalette(mode, pulse);

    // Color shifting using sine time offsets for red/blue
    palColor.r *= sin(time * redShift + length(p)) * 0.5 + 0.5;
    palColor.b *= cos(time * blueShift + length(p)) * 0.5 + 0.5;

    // Apply texture as a color influence
    vec3 texColor = texture2D(Texture, gl_FragCoord.xy / RENDERSIZE.xy * TextureScale).rgb;
    vec3 base = vec3(ot.x, ot.y * 0.5 + ot.x * 0.3, ot.y) + m * 0.2;
    vec3 finalColor = base * mix(palColor, texColor, TextureMix);

    return finalColor;
}

// --- Main Function with Full Pipeline ---
void main() {
    vec2 uv = (gl_FragCoord.xy - RENDERSIZE.xy * 0.5) / RENDERSIZE.y;
    vec2 d = vec2(0.0, 0.5) / RENDERSIZE.xy;

    float t = TIME * Speed;

    // Apply texture warping to the UVs before rendering
    vec3 warp = texture2D(Texture, gl_FragCoord.xy / RENDERSIZE.xy * TextureScale).rgb;
    uv += (warp.xy - 0.5) * TextureWarp;

    vec3 col = render(uv, t, FractalScale, MorphAmount, Pulse, int(ColorPaletteMode), RedShiftSpeed, BlueShiftSpeed, GeometryDeform, TrapBias);

    // Post-processing effects (from previous shader)
    col = (col - 0.5) * Contrast + 0.5;
    col *= Brightness;
    col += Glow * (vec3(1.0) - col) * col;

    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
