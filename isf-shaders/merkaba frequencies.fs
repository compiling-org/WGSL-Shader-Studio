/*
{
    "CATEGORIES": [
        "Generator",
        "Fractal",
        "Optimized",
        "Volumetric",
        "Psychedelic"
    ],
    "CREDIT": "Converted from ShaderToy by phreax (2023) + merged features by Unity Shader Expert",
    "DESCRIPTION": "Fractal morphing shader extended with texture influence, chaotic transforms, symmetry, volumetric glow and palette morphing.",
    "ISFVSN": "2",
    "INPUTS": [
        { "NAME": "speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0 },
        { "NAME": "zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 3.0 },
        { "NAME": "morph", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "foldCount", "TYPE": "float", "DEFAULT": 5.0, "MIN": 2.0, "MAX": 12.0 },
        { "NAME": "fractalPower", "TYPE": "float", "DEFAULT": 2.2, "MIN": 1.0, "MAX": 6.0 },
        { "NAME": "spacing", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.05, "MAX": 1.0 },
        { "NAME": "orbit", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "shake", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "glitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "contrast", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "brightness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "palette", "TYPE": "float", "DEFAULT": 2.5, "MIN": 0.0, "MAX": 6.0 },
        { "NAME": "transformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0.0, "MAX": 5.0 },
        { "NAME": "geometryType", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 6.0 },
        { "NAME": "chaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "chaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0 },
        { "NAME": "glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "symmetry", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 4.0 },
        { "NAME": "chaosMix", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0 },
        { "NAME": "sharpness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0 },
        { "NAME": "falloffCurve", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.1, "MAX": 3.0 },
        { "NAME": "cameraOrbit", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14 },
        { "NAME": "cameraPitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.57, "MAX": 1.57 },
        { "NAME": "cameraRoll", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14 },
        { "NAME": "focusNear", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5.0, "MAX": 5.0 },
        { "NAME": "focusFar", "TYPE": "float", "DEFAULT": 2.6, "MIN": 0.1, "MAX": 10.0 },
        { "NAME": "fov", "TYPE": "float", "DEFAULT": 1.6, "MIN": 0.2, "MAX": 3.0 },
        { "NAME": "stepCount", "TYPE": "float", "DEFAULT": 6.0, "MIN": 1.0, "MAX": 60.0 },
        { "NAME": "Texture", "TYPE": "image" },
        { "NAME": "textureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 },
        { "NAME": "textureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0 },
        { "NAME": "displace", "TYPE": "float", "DEFAULT": 0.04, "MIN": 0.01, "MAX": 0.1 },
        { "NAME": "shimmer", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 }
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

mat2 rot(float a) { return mat2(cos(a), -sin(a), sin(a), cos(a)); }

vec3 bump3y(vec3 x, vec3 yoffset) {
    vec3 y = 1. - x * x;
    return clamp(y - yoffset, vec3(0), vec3(1));
}

vec3 getPalette(float t, float id) {
    vec3 p0 = vec3(sin(t*2.1), sin(t*3.7+0.5), sin(t*5.3+1.0));
    vec3 p1 = vec3(0.5+0.5*sin(t*PI*2.0), 0.5+0.5*cos(t*PI*1.5), 0.5+0.5*sin(t*PI*0.5));
    vec3 p2 = vec3(sin(t*8.0), cos(t*6.0), sin(t*4.0+1.5));
    vec3 p3 = vec3(sin(t*2.0), sin(t*5.0 + 0.5), cos(t*3.5 + 1.2));
    vec3 p4 = vec3(0.8, 0.6, 1.0)*vec3(sin(t*2.0), sin(t*3.0), cos(t*4.0));
    vec3 p5 = bump3y(vec3(3.5,2.9,2.4)*(t-vec3(0.7,0.5,0.3)), vec3(0.1));
    vec3 p6 = bump3y(vec3(3.9,3.2,3.9)*(t-vec3(0.1,0.8,0.6)), vec3(0.7));
    vec3 a = mix(p0, p1, smoothstep(0.,1.,fract(id)));
    vec3 b = mix(p2, p3, smoothstep(0.,1.,fract(id-2.)));
    vec3 c = mix(p4, mix(p5, p6, 0.5), smoothstep(0.,1.,fract(id-4.)));
    if(id < 2.0) return a;
    if(id < 4.0) return b;
    return c;
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

vec2 foldSym(vec2 p, float N, float spacing) {
    float t = atan(p.x, -p.y);
    t = mod(t + PI / N, 2.0 * PI / N) - PI / N;
    p = length(p) * vec2(cos(t), sin(t));
    p = abs(p) - spacing;
    p = abs(p) - spacing;
    return p;
}

float shapeSpikeFractal(vec3 p, int geo, float chaos, float mixAmt) {
    float d = 0.0;
    
    float base;
    if (geo == 0) base = length(p) - 1.0;
    else if (geo == 1) {
        vec2 q = vec2(length(p.xz)-1.0, p.y);
        base = length(q) - 0.3;
    } else {
        vec3 p_copy = p;
        for (int i = 0; i < 128; i++) {
            if (i >= int(stepCount)) break;
            p_copy = abs(p_copy) / dot(p_copy, p_copy + 0.001) - 0.5;
            p_copy *= 0.95;
            d += length(p_copy);
        }
        base = d / 20.0;
        // The original recursive call was here. It has been removed.
        // The fractal logic for geo 2, 3 etc. is now handled by the loop above.
    }
    
    float chaos_val = (sin(p.x*3. + TIME*chaosSpeed) + sin(p.y*4. + TIME*chaosSpeed*1.2) + sin(p.z*5. + TIME*chaosSpeed*0.8)) * chaos;

    return mix(base, chaos_val, mixAmt);
}

vec3 renderPass(float time_val) {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
    uv *= zoom;
    
    if (shake > 0.0) {
        uv += vec2(sin(uv.y * 40. + time_val * 5.), cos(uv.x * 40. - time_val * 3.)) * 0.003 * shake;
    }

    vec3 ro = vec3(0.0, 0.0, -3.0);
    vec3 rd = normalize(vec3(uv, 1.0) * fov);

    float co = cos(cameraOrbit), so = sin(cameraOrbit);
    float cp = cos(cameraPitch), sp = sin(cameraPitch);
    float cr = cos(cameraRoll), sr = sin(cameraRoll);
    mat3 cameraMat = mat3(
        co * cr + so * sp * sr, sr * cp, -so * cr + co * sp * sr,
        -co * sr + so * sp * cr, cr * cp, sr * so + co * sp * cr,
        so * cp, -sp, co * cp
    );
    rd = cameraMat * rd;

    float orbitSpeed = orbit * 2.0;
    vec3 a = normalize(vec3(1, 3, 5));
    rd = R(rd, a, time_val * orbitSpeed);
    
    vec3 warp = triplanarTexture(ro * textureScale, 1.0) - 0.5;
    vec3 roWarped = ro + warp * textureWarp;
    
    vec3 col = vec3(0.0);
    float dist = 0.0;
    
    for (int i = 0; i < MAX_STEPS; i++) {
        vec3 p = roWarped + dist * rd;
        
        p.xy = foldSym(p.xy, foldCount, spacing);
        p.yz *= rot(0.38 * 2.0 * PI);
        
        p *= max(symmetry, 0.001);
        int mode = int(transformMode);
        if (mode == 1) p = abs(p);
        else if (mode == 2) p += sin(p * 3.0 + time_val * chaosSpeed) * chaosIntensity * 0.3;
        else if (mode == 3) {
            p += sin(p * (1.0 + chaosIntensity * 2.0) + time_val * chaosSpeed) * chaosIntensity * 0.5;
            p = fract(p * 1.5) - 0.75;
        }
        if (mode == 4 || mode == 5) {
            float angle = atan(p.z, p.x);
            float r = length(p.xz);
            float spin = time_val * chaosSpeed * (mode == 4 ? 0.2 : 0.3);
            angle += spin;
            p.x = cos(angle) * r;
            p.z = sin(angle) * r;
        }

        float d = shapeSpikeFractal(p, int(geometryType), chaosIntensity, chaosMix);
        d = max(abs(d), 0.01);

        float fade = exp(-float(i)*0.03*sharpness);
        float focus = smoothstep(focusNear, focusFar, dist);
        
        vec3 palCol = getPalette(dist * 0.8 + 0.15 * time_val * speed, palette);
        vec3 texCol = triplanarTexture(p * textureScale, 1.0);
        float b = 0.005 / (0.01 + d * falloffCurve);

        col += mix(palCol, texCol, 0.5) * b * fade * focus;

        dist += d;
        if (dist > BAILOUT) break;
    }
    
    float shimmer_noise = (fract(sin(dot(gl_FragCoord.xy, vec2(12.9898,78.233))) * 43758.5453) * 2.0 - 1.0) * shimmer;
    col += shimmer_noise * 0.1;
    
    if (glitch > 0.0) {
        vec2 moire_noise = sin(uv * 50.0 + time_val * 5.0);
        vec3 overlay = getPalette(dot(moire_noise, vec2(1.0, 1.5)) * 2.0, palette + 2.0);
        col = mix(col, col + overlay * 0.4, glitch);
    }
    
    col *= glow;
    col = (col - 0.5) * contrast + 0.5;
    col *= brightness;
    
    return col;
}

void main() {
    vec3 finalColor;

    finalColor.r = renderPass(TIME).r;
    finalColor.g = renderPass(TIME + displace).g;
    finalColor.b = renderPass(TIME + 2.0 * displace).b;
    
    finalColor = pow(finalColor * 1.2, vec3(1.1));
    finalColor = mix(vec3(0.5), finalColor, contrast);
    finalColor = mix(vec3(dot(finalColor, vec3(0.333))), finalColor, saturation);

    gl_FragColor = vec4(clamp(finalColor, 0.0, 1.0), 1.0);
}