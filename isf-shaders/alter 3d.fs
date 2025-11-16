/*{
    "CATEGORIES": [
        "Psychedelic",
        "Geometry",
        "Transform",
        "Chaos"
    ],
    "DESCRIPTION": "A psychedelic fractal exploring new 3D fractals and geometries with dynamic and complex transformations. This version focuses on Kleinian-like inversions and recursive folding patterns with added surface details like bubbles and spores.",
    "ISFVSN": "2",
    "PASSES": [
        {}
    ],
    "INPUTS": [
        {"NAME": "Speed", "TYPE": "float", "DEFAULT": 10.0, "MIN": 0.1, "MAX": 50},
        {"NAME": "Zoom", "TYPE": "float", "DEFAULT": 1.4, "MIN": 0.5, "MAX": 3},
        {"NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1, "MIN": 0, "MAX": 5, "VALUES": [{"NAME":"Basic", "VALUE":0},{"NAME":"Mobius", "VALUE":1},{"NAME":"Inversion", "VALUE":2},{"NAME":"FoldRotate", "VALUE":3},{"NAME":"Hyperbolic", "VALUE":4},{"NAME":"ShearTwist", "VALUE":5}]},
        {"NAME": "GeometryType", "TYPE": "float", "DEFAULT": 0, "MIN": 0, "MAX": 9, "VALUES": [{"NAME":"Sphere", "VALUE":0},{"NAME":"KleinianFractal", "VALUE":1},{"NAME":"BoxFolding", "VALUE":2},{"NAME":"HexPrism", "VALUE":3},{"NAME":"TorusKnot", "VALUE":4},{"NAME":"Octahedron", "VALUE":5},{"NAME":"Mandelbox", "VALUE":6},{"NAME":"Kaliset", "VALUE":7},{"NAME":"TorusTunnel", "VALUE":8},{"NAME":"Chaos", "VALUE":9}]},
        {"NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0, "MAX": 2},
        {"NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.20, "MIN": 0.1, "MAX": 4},
        {"NAME": "ColorPaletteMode", "TYPE": "float", "DEFAULT": 14, "MIN": 0, "MAX": 19, "VALUES": [
            {"NAME":"Luminance", "VALUE":0},{"NAME":"Hydro", "VALUE":1},{"NAME":"Toxic", "VALUE":2},{"NAME":"Chrono", "VALUE":3},{"NAME":"Vapor", "VALUE":4},
            {"NAME":"Acidic", "VALUE":5},{"NAME":"DeepSpace", "VALUE":6},{"NAME":"Spectrum", "VALUE":7},{"NAME":"Plasma", "VALUE":8},
            {"NAME":"Aurora", "VALUE":9},{"NAME":"Synthwave", "VALUE":10},{"NAME":"CosmicDust", "VALUE":11},{"NAME":"Volcano", "VALUE":12},
            {"NAME":"Emerald", "VALUE":13},{"NAME":"Magma", "VALUE":14},{"NAME":"Cyberpunk", "VALUE":15},{"NAME":"Jungle", "VALUE":16},
            {"NAME":"Gothic", "VALUE":17},{"NAME":"Electric", "VALUE":18},{"NAME":"Ice", "VALUE":19}
        ]},
        {"NAME": "Brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0, "MAX": 3},
        {"NAME": "Contrast", "TYPE": "float", "DEFAULT": 1.3, "MIN": 0.1, "MAX": 3},
        {"NAME": "Glow", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0, "MAX": 2},
        {"NAME": "Symmetry", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0, "MAX": 4},
        {"NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0, "MAX": 1},
        {"NAME": "CamPosX", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5, "MAX": 5},
        {"NAME": "CamPosY", "TYPE": "float", "DEFAULT": 0.0, "MIN": -5, "MAX": 5},
        {"NAME": "CamPosZ", "TYPE": "float", "DEFAULT": -3.0, "MIN": -10, "MAX": 1},
        {"NAME": "CamFOV", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.5, "MAX": 2.0},
        {"NAME": "TunnelMotion", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0, "MAX": 1, "VALUES":[{"NAME":"Off", "VALUE":0.0},{"NAME":"On", "VALUE":1.0}]},
        {"NAME": "Sharpness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0},
        {"NAME": "FocusNear", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 10.0},
        {"NAME": "FocusFar", "TYPE": "float", "DEFAULT": 5.0, "MIN": 0.0, "MAX": 20.0},
        {"NAME": "Texture", "TYPE": "image"},
        {"NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0, "MAX": 2},
        {"NAME": "StepCount", "TYPE": "float", "DEFAULT": 128, "MIN": 32, "MAX": 256},
        {"NAME": "FalloffCurve", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0},
        {"NAME": "SurfaceDetailScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0, "MAX": 5.0},
        {"NAME": "BubbleSize", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.01, "MAX": 1},
        {"NAME": "SporeIntensity", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 1},
        {"NAME": "LightDirectionX", "TYPE": "float", "DEFAULT": 0.5, "MIN": -1, "MAX": 1},
        {"NAME": "LightDirectionY", "TYPE": "float", "DEFAULT": 0.5, "MIN": -1, "MAX": 1},
        {"NAME": "LightDirectionZ", "TYPE": "float", "DEFAULT": 0.5, "MIN": -1, "MAX": 1}
    ]
}
*/
// Define constants
#define PI 3.141592
#define BAILOUT 16.0
#define EPSILON 0.001

// Rotation matrix for 2D
mat2 rot2(float a) {
    float c= cos(a);
    float s= sin(a);
    return mat2(c,s,-s,c);
}

// Palette function
vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
    return a + b * cos(6.28318 * (c * t + d));
}

// Color palettes
vec3 getColorPalette(int mode, float t) {
    if (mode==0) return pal(t, vec3(0.5,0.5,0.5), vec3(0.5,0.5,0.5), vec3(1.0,1.0,1.0), vec3(0.0,0.33,0.67)); // Luminance
    if (mode==1) return pal(t, vec3(0.1,0.5,0.9), vec3(0.1,0.1,0.8), vec3(0.1,0.2,0.9), vec3(0.1,0.3,0.8)); // Hydro
    if (mode==2) return pal(t, vec3(0.2,1.0,0.2), vec3(0.5,0.9,0.2), vec3(0.8,0.8,0.2), vec3(0.3,0.6,0.3)); // Toxic
    if (mode==3) return pal(t, vec3(1.0,0.5,0.0), vec3(0.0,0.5,1.0), vec3(0.5,0.0,1.0), vec3(0.5,0.5,0.5)); // Chrono
    if (mode==4) return pal(t, vec3(0.8,0.7,0.9), vec3(0.6,0.5,0.8), vec3(0.9,0.8,1.0), vec3(0.2,0.4,0.6)); // Vapor
    if (mode==5) return pal(t, vec3(0.0,1.0,0.1), vec3(0.8,0.2,0.5), vec3(0.9,0.5,0.2), vec3(0.2,0.4,0.6)); // Acidic
    if (mode==6) return pal(t, vec3(0.0,0.0,0.1), vec3(0.0,0.1,0.2), vec3(0.0,0.2,0.3), vec3(0.0,0.3,0.4)); // DeepSpace
    if (mode==7) return pal(t, vec3(1.0,0.0,0.0), vec3(0.0,1.0,0.0), vec3(0.0,0.0,1.0), vec3(1.0,1.0,1.0)); // Spectrum
    if (mode==8) return pal(t, vec3(0.8,0.2,0.2), vec3(0.2,0.8,0.2), vec3(0.2,0.2,0.8), vec3(0.2,0.4,0.6)); // Plasma
    if (mode==9) return pal(t, vec3(0.0,1.0,0.5), vec3(1.0,0.0,0.5), vec3(0.5,1.0,0.0), vec3(0.2,0.4,0.6)); // Aurora
    if (mode==10) return pal(t, vec3(1.0,0.0,0.8), vec3(0.0,1.0,0.5), vec3(0.5,0.0,1.0), vec3(1.0,0.5,0.0)); // Synthwave
    if (mode==11) return pal(t, vec3(0.6,0.1,0.7), vec3(0.2,0.3,0.8), vec3(0.9,0.4,0.2), vec3(0.7,0.7,0.7)); // CosmicDust
    if (mode==12) return pal(t, vec3(1.0,0.1,0.0), vec3(0.8,0.3,0.1), vec3(0.6,0.2,0.0), vec3(0.2,0.4,0.6)); // Volcano
    if (mode==13) return pal(t, vec3(0.1,0.5,0.2), vec3(0.2,0.7,0.3), vec3(0.3,0.9,0.4), vec3(0.2,0.4,0.6)); // Emerald
    if (mode==14) return pal(t, vec3(1.0,0.3,0.0), vec3(0.8,0.2,0.1), vec3(0.6,0.1,0.0), vec3(0.2,0.4,0.6)); // Magma
    if (mode==15) return pal(t, vec3(0.0,0.8,1.0), vec3(0.5,0.0,0.5), vec3(0.1,0.9,0.5), vec3(0.3,0.5,0.7)); // Cyberpunk
    if (mode==16) return pal(t, vec3(0.0,0.4,0.1), vec3(0.2,0.6,0.2), vec3(0.4,0.8,0.3), vec3(0.1,0.3,0.2)); // Jungle
    if (mode==17) return pal(t, vec3(0.1,0.0,0.1), vec3(0.2,0.1,0.2), vec3(0.4,0.2,0.4), vec3(0.1,0.1,0.1)); // Gothic
    if (mode==18) return pal(t, vec3(1.0,0.0,1.0), vec3(0.0,1.0,1.0), vec3(1.0,1.0,0.0), vec3(0.2,0.4,0.6)); // Electric
    if (mode==19) return pal(t, vec3(0.0,0.5,1.0), vec3(1.0,1.0,0.0), vec3(0.5,0.0,0.0), vec3(0.2,0.4,0.6)); // Ice
    return pal(t, vec3(0.5), vec3(0.5), vec3(0.5), vec3(0.0));
}

// Mandelbox SDF
float shapeMandelbox(vec3 p) {
    vec3 c = p;
    float scale = 2.0;
    float r = 1.0;
    for (int i = 0; i < 8; ++i) {
        p = clamp(p, -2.0, 2.0) * 2.0 - p;
        float r2 = dot(p, p);
        if (r2 < 0.5) {
            p *= 2.0;
            r *= 2.0;
        } else if (r2 > 1.0) {
            p /= r2;
            r /= r2;
        }
        p = p * scale + c;
        r = r * abs(scale) + 1.0;
        if (length(p) > BAILOUT) break;
    }
    return length(p) / abs(r);
}

// Kaliset SDF
float shapeKaliset(vec3 p) {
    vec3 c = p;
    float r = 1.0;
    for (int i = 0; i < 10; i++) {
        p = 2.0 * clamp(p, -1.0, 1.0) - p;
        p = abs(p);
        float r2 = dot(p, p);
        p = p * 2.0 / r2 + c;
        r = r * 2.0 / r2;
        if (length(p) > BAILOUT) break;
    }
    return length(p) / r;
}

// Kleinian Fractal / Sphere Inversion
vec3 kleinianInversion(vec3 p) {
    p = abs(p);
    float r = 1.0;
    for (int i = 0; i < 4; i++) {
        p /= dot(p, p);
        p = p * 2.0 - vec3(1.0);
        r *= 2.0;
    }
    return (p / r) - 1.0;
}

// Box Folding - Sierpinski-like 3D mods
float shapeBoxFolding(vec3 p) {
    float s = 1.5;
    float r = 1.0;
    for(int i = 0; i < 4; i++) {
        p = abs(p) * s - vec3(1.0);
        s *= 1.5;
        r *= 1.5;
    }
    return (length(p) - 1.0) / r;
}

// Hexagonal Prism SDF
float shapeHexPrism(vec3 p) {
    const vec3 k = vec3(-0.866025404, 0.5, 0.577350269);
    p = abs(p);
    p.xy -= 2.0*min(dot(k.xy, p.xy), 0.0)*k.xy;
    vec2 d = vec2(length(p.xy-vec2(clamp(p.x, -k.z, k.z), 0.0))-0.5, p.z-0.5);
    return min(max(d.x,d.y),0.0) + length(max(d,0.0));
}

// Torus Knot SDF
float shapeTorusKnot(vec3 p) {
    float r1 = 0.5, r2 = 0.2;
    float a1 = 3.0, a2 = 2.0;
    float t = TIME * 0.1;
    float c = cos(a1*t), s = sin(a1*t);
    mat2 m = mat2(c,-s,s,c);
    p.xz = m * p.xz;
    vec2 q = vec2(length(p.xz) - r1, p.y);
    return length(q) - r2;
}

// Chaos shape - a different version
float shapeChaos(vec3 p, float chaos) {
    float c = sin(p.x * 5.0 + TIME * ChaosSpeed * 1.5) + cos(p.y * 6.0 + TIME * ChaosSpeed) + sin(p.z * 4.0 + TIME * ChaosSpeed * 0.7);
    return c * chaos;
}

// Torus Tunnel SDF
float shapeTorusTunnel(vec3 p) {
    vec2 q = vec2(length(p.xz)-1.0,p.y);
    return length(q)-0.1;
}

// A local scene function to be called within the normal calculation
float localScene(vec3 p, int geomType, float chaos, float chaosMix) {
    float baseDist = 0.0;
    if (geomType == 0) baseDist = length(p) - 1.0; // Sphere
    else if (geomType == 1) baseDist = length(kleinianInversion(p));
    else if (geomType == 2) baseDist = shapeBoxFolding(p);
    else if (geomType == 3) baseDist = shapeHexPrism(p);
    else if (geomType == 4) baseDist = shapeTorusKnot(p);
    else if (geomType == 5) baseDist = length(abs(p) - 1.0); // Octahedron
    else if (geomType == 6) baseDist = shapeMandelbox(p);
    else if (geomType == 7) baseDist = shapeKaliset(p);
    else if (geomType == 8) baseDist = shapeTorusTunnel(p);
    else if (geomType == 9) baseDist = shapeChaos(p, chaos);

    float chaosDist = shapeChaos(p, chaos);
    return mix(baseDist, chaosDist, chaosMix);
}

// New: Spore/Grain noise function
float noise(vec3 p) {
    vec3 ip = floor(p);
    vec3 fp = fract(p);
    fp = fp * fp * (3.0 - 2.0 * fp);
    vec2 uv = (ip.xy + vec2(37.0, 17.0) * ip.z) + fp.xy;
    vec2 pos = uv / 1024.0;
    return 2.0 * texture(Texture, pos).x - 1.0;
}

// New: Function to add surface details (bubbles and spores)
float addSurfaceDetails(vec3 p, float baseDist, float scale, float bubbleSize, float sporeIntensity) {
    float detailDist = baseDist;
    if (scale > 0.0) {
        // Add bubbles
        vec3 p_bubbles = p * scale;
        float bubbleDist = length(p_bubbles - floor(p_bubbles + 0.5)) - bubbleSize;
        detailDist = max(detailDist, -bubbleDist); // Use max for subtraction
        
        // Add spores/grains
        float sporeDist = noise(p * scale * 5.0) * sporeIntensity;
        detailDist = detailDist - sporeDist;
    }
    return detailDist;
}

// Main Scene Function
float scene(vec3 p, int geomType, float chaos, float chaosMix, float surfaceDetailScale, float bubbleSize, float sporeIntensity) {
    float baseDist = localScene(p, geomType, chaos, chaosMix);
    return addSurfaceDetails(p, baseDist, surfaceDetailScale, bubbleSize, sporeIntensity);
}

// New: Function to calculate the normal of the surface
vec3 getNormal(vec3 p, int geomType, float chaos, float chaosMix, float surfaceDetailScale, float bubbleSize, float sporeIntensity) {
    vec2 e = vec2(1.0, -1.0) * 0.5773;
    const float normalEpsilon = 0.001;
    return normalize(e.xyy * scene(p + e.xyy * normalEpsilon, geomType, chaos, chaosMix, surfaceDetailScale, bubbleSize, sporeIntensity) + 
                     e.yyx * scene(p + e.yyx * normalEpsilon, geomType, chaos, chaosMix, surfaceDetailScale, bubbleSize, sporeIntensity) + 
                     e.yxy * scene(p + e.yxy * normalEpsilon, geomType, chaos, chaosMix, surfaceDetailScale, bubbleSize, sporeIntensity) + 
                     e.xxx * scene(p + e.xxx * normalEpsilon, geomType, chaos, chaosMix, surfaceDetailScale, bubbleSize, sporeIntensity));
}

// Transformation Function
vec3 applyTransform(vec3 p, int mode, float chaos, float symmetry, float chaosspeed) {
    p = p * symmetry;
    if (mode == 1) {
        // Mobius transform
        float r2 = dot(p, p);
        if (r2 > 1.0) p /= r2;
        p += vec3(sin(TIME * chaosspeed), cos(TIME * chaosspeed), sin(TIME * chaosspeed * 0.5)) * chaos * 0.5;
    } else if (mode == 2) {
        // Inversion
        p = p / dot(p, p) * chaos;
        p += sin(p * 2.0 + TIME * chaosspeed) * 0.1;
    } else if (mode == 3) {
        // Fold and Rotate
        p.xy = rot2(p.z * 0.5 + TIME * chaosspeed) * p.xy;
        p.yz = rot2(p.x * 0.5 + TIME * chaosspeed * 1.2) * p.yz;
    } else if (mode == 4) {
        // Hyperbolic
        float h = p.x * p.x - p.y * p.y - p.z * p.z;
        p.z += sin(h + TIME * chaosspeed) * chaos;
        p.x += cos(h + TIME * chaosspeed * 0.7) * chaos;
    } else if (mode == 5) {
        // Shear and Twist
        p.y += sin(p.x + TIME*chaosspeed)*chaos;
        p.z += cos(p.y + TIME*chaosspeed)*chaos;
        p.xy = rot2(p.z * 0.5 + TIME * chaosspeed * 0.8) * p.xy;
    }
    return p;
}

// Main Fragment Shader
void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
    float t = TIME * Speed;

    vec3 origin = vec3(CamPosX, CamPosY, CamPosZ);
    vec3 dir = normalize(vec3(uv * Zoom * CamFOV, 1.0));

    // Optional tunnel motion
    if (TunnelMotion > 0.5) {
        float tunnelSpeed = 0.5;
        origin = vec3(0.0, 0.0, 1.0) * TIME * tunnelSpeed;
        dir = normalize(vec3(uv * Zoom * CamFOV, 1.0));
    }
    
    vec3 color = vec3(0.0);
    float dist = 0.0;
    
    vec3 currentPos = origin;
    float hitDistance = -1.0;
    vec3 hitPos = vec3(0.0);
    
    int geomType = int(GeometryType);
    int mode = int(TransformMode);
    float chaos = ChaosIntensity;
    float chaosspeed = ChaosSpeed;
    float symmetry = Symmetry;
    float chaosMix = ChaosMix;
    float brightness = Brightness;
    float contrast = Contrast;
    float glow = Glow;
    float stepCount = StepCount;
    float falloffCurve = FalloffCurve;
    float sharpness = Sharpness;
    float focusNear = FocusNear;
    float focusFar = FocusFar;
    float surfaceDetailScale = SurfaceDetailScale;
    float bubbleSize = BubbleSize;
    float sporeIntensity = SporeIntensity;

    for (int i = 0; i < int(stepCount); i++) {
        vec3 p = origin + dir * dist;
        p = applyTransform(p, mode, chaos, symmetry, chaosspeed);
        
        float d = scene(p, geomType, chaos, chaosMix, surfaceDetailScale, bubbleSize, sporeIntensity);
        
        dist += d;
        
        if (d < EPSILON || dist > BAILOUT) {
            hitDistance = dist;
            hitPos = p;
            break;
        }
    }

    if (hitDistance > 0.0) {
        vec3 lightDir = normalize(vec3(LightDirectionX, LightDirectionY, LightDirectionZ));
        vec3 normal = getNormal(hitPos, geomType, chaos, chaosMix, surfaceDetailScale, bubbleSize, sporeIntensity);
        float diffuse = max(dot(normal, lightDir), 0.0);
        
        // Base color from palette
        vec3 baseColor = getColorPalette(int(ColorPaletteMode), hitPos.z + t * 0.2);

        // Psychedelic pulse
        float pulse = sin(TIME * ChaosSpeed) * 0.5 + 0.5;
        baseColor *= 1.0 + 0.3 * pulse;

        // Apply lighting
        baseColor *= diffuse + 0.2; // Ambient light
        
        // Apply texture
        vec2 texCoord = hitPos.xy * 0.5 + 0.5;
        texCoord += sin(hitPos.z * TextureWarp) * 0.2;
        vec3 textureColor = texture(Texture, texCoord).rgb;
        baseColor = mix(baseColor, textureColor, 0.5);

        // Apply Depth of Field (DoF)
        float blur = 0.0;
        if (hitDistance < focusNear) {
            blur = (focusNear - hitDistance) / focusNear;
        } else if (hitDistance > focusFar) {
            blur = (hitDistance - focusFar) / (BAILOUT - focusFar);
        }
        blur = clamp(blur, 0.0, 1.0) * (1.0 - sharpness);
        baseColor = mix(baseColor, vec3(0.5), blur);
        
        color = baseColor;
    } else {
        color = vec3(0.0);
    }
    
    // Final post-processing
    color = (color - 0.5) * contrast + 0.5;
    color *= brightness;
    color *= glow;

    gl_FragColor = vec4(color, 1.0);
}