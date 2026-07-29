/*{
  "DESCRIPTION": "Merged: original tunnel fractal core + selectable liminal/chaos/geometry modes, fixed undefined vars, black/backgroundColor background, working sliders. Transforms are clamped so geometry stays within the tunnel.",
  "CATEGORIES": ["Fractal","Raymarching","Tunnel","Volumetric","Psychedelic"],
  "ISFVSN": "2.0",
  "INPUTS":[
    {"NAME":"masterSpeed","TYPE":"float","DEFAULT":1.0,"MIN":0.1,"MAX":5.0},
    {"NAME":"globalGlowIntensity","TYPE":"float","DEFAULT":1.0,"MIN":0.0,"MAX":5.0},
    {"NAME":"paletteSelect","TYPE":"float","DEFAULT":0.0,"MIN":0.0,"MAX":19.0},
    {"NAME":"paletteAnimSpeed","TYPE":"float","DEFAULT":0.05,"MIN":0.0,"MAX":0.5},
    {"NAME":"paletteBrightness","TYPE":"float","DEFAULT":1.0,"MIN":0.1,"MAX":5.0},

    {"NAME":"fractalGlowStrength","TYPE":"float","DEFAULT":0.075,"MIN":0.0,"MAX":0.5},

    {"NAME":"pulseLineIntensity","TYPE":"float","DEFAULT":0.0,"MIN":0.0,"MAX":1.0},
    {"NAME":"pulseLineSpeed","TYPE":"float","DEFAULT":1.0,"MIN":0.0,"MAX":5.0},
    {"NAME":"pulseLineThickness","TYPE":"float","DEFAULT":0.02,"MIN":0.001,"MAX":0.1},
    {"NAME":"pulseLineDirection","TYPE":"float","DEFAULT":0.0,"MIN":0.0,"MAX":3.0},

    {"NAME":"enableAutoCameraMovement","TYPE":"bool","DEFAULT":true},
    {"NAME":"camX","TYPE":"float","DEFAULT":0.0,"MIN":-10.0,"MAX":10.0},
    {"NAME":"camY","TYPE":"float","DEFAULT":0.0,"MIN":-10.0,"MAX":10.0},
    {"NAME":"camZ","TYPE":"float","DEFAULT":0.0,"MIN":-10.0,"MAX":10.0},
    {"NAME":"lookAtX","TYPE":"float","DEFAULT":0.0,"MIN":-5.0,"MAX":5.0},
    {"NAME":"lookAtY","TYPE":"float","DEFAULT":0.0,"MIN":-5.0,"MAX":5.0},
    {"NAME":"lookAtZ","TYPE":"float","DEFAULT":1.0,"MIN":-5.0,"MAX":5.0},
    {"NAME":"camFOV","TYPE":"float","DEFAULT":1.0,"MIN":0.1,"MAX":5.0},
    {"NAME":"autoCamSpeed","TYPE":"float","DEFAULT":1.0,"MIN":0.1,"MAX":5.0},

    {"NAME":"P_X_CosFactor","TYPE":"float","DEFAULT":0.3,"MIN":0.05,"MAX":1.0},
    {"NAME":"P_X_Amp","TYPE":"float","DEFAULT":0.5,"MIN":0.0,"MAX":2.0},
    {"NAME":"P_X_Range","TYPE":"float","DEFAULT":6.0,"MIN":1.0,"MAX":20.0},
    {"NAME":"P_Y_CosFactor","TYPE":"float","DEFAULT":0.4,"MIN":0.05,"MAX":1.0},
    {"NAME":"P_Y_Amp","TYPE":"float","DEFAULT":0.3,"MIN":0.0,"MAX":2.0},
    {"NAME":"P_Y_Range","TYPE":"float","DEFAULT":16.0,"MIN":1.0,"MAX":30.0},

    {"NAME":"cameraRotSpeed","TYPE":"float","DEFAULT":0.3,"MIN":0.0,"MAX":1.0},
    {"NAME":"cameraRotAmount","TYPE":"float","DEFAULT":0.6,"MIN":0.0,"MAX":1.0},
    {"NAME":"cameraRotZOffsetSpeed","TYPE":"float","DEFAULT":22.0,"MIN":0.0,"MAX":50.0},
    {"NAME":"cameraRotZOffsetAmount","TYPE":"float","DEFAULT":3.0,"MIN":0.0,"MAX":10.0},

    {"NAME":"orbAmplitudeX","TYPE":"float","DEFAULT":2.6,"MIN":0.1,"MAX":5.0},
    {"NAME":"orbAmplitudeY","TYPE":"float","DEFAULT":2.125,"MIN":0.1,"MAX":5.0},
    {"NAME":"orbSpeedX","TYPE":"float","DEFAULT":1.0,"MIN":0.1,"MAX":5.0},
    {"NAME":"orbSpeedY","TYPE":"float","DEFAULT":2.5,"MIN":0.1,"MAX":5.0},
    {"NAME":"orbTanCosAmplitude","TYPE":"float","DEFAULT":0.3,"MIN":0.0,"MAX":1.0},
    {"NAME":"orbTanCosSpeed","TYPE":"float","DEFAULT":0.3,"MIN":0.1,"MAX":1.0},

    {"NAME":"tunnelRaymarchSteps","TYPE":"float","DEFAULT":60.0,"MIN":10.0,"MAX":200.0},
    {"NAME":"fractalRaymarchSteps","TYPE":"float","DEFAULT":60.0,"MIN":10.0,"MAX":200.0},
    {"NAME":"raymarchMinDist","TYPE":"float","DEFAULT":0.002,"MIN":0.0001,"MAX":0.01},
    {"NAME":"raymarchMaxDist","TYPE":"float","DEFAULT":100.0,"MIN":10.0,"MAX":500.0},

    {"NAME":"tunnelDensityOffset","TYPE":"float","DEFAULT":0.5,"MIN":0.0,"MAX":2.0},
    {"NAME":"tunnelDensityFactor","TYPE":"float","DEFAULT":0.75,"MIN":0.0,"MAX":2.0},
    {"NAME":"tunnelDensitySpeed","TYPE":"float","DEFAULT":0.3,"MIN":0.0,"MAX":1.0},

    {"NAME":"fractalIterations","TYPE":"float","DEFAULT":9.0,"MIN":1.0,"MAX":20.0},
    {"NAME":"fractalAmp","TYPE":"float","DEFAULT":1.6,"MIN":0.1,"MAX":5.0},
    {"NAME":"fractalOffset","TYPE":"float","DEFAULT":1.0,"MIN":0.0,"MAX":2.0},

    {"NAME":"flashingLightIntensity","TYPE":"float","DEFAULT":0.5,"MIN":0.0,"MAX":1.0},
    {"NAME":"flashingLightSpeed","TYPE":"float","DEFAULT":2.0,"MIN":0.1,"MAX":10.0},
    {"NAME":"flashingLightMixAmount","TYPE":"float","DEFAULT":0.5,"MIN":0.0,"MAX":1.0},
    {"NAME":"flashingLightDotVec","TYPE":"float","DEFAULT":4.0,"MIN":0.1,"MAX":10.0},

    {"NAME":"shimmerStrength","TYPE":"float","DEFAULT":0.0,"MIN":0.0,"MAX":1.0},
    {"NAME":"shimmerSpeed","TYPE":"float","DEFAULT":5.0,"MIN":0.1,"MAX":20.0},
    {"NAME":"shakeAmount","TYPE":"float","DEFAULT":0.0,"MIN":0.0,"MAX":0.1},
    {"NAME":"shakeSpeed","TYPE":"float","DEFAULT":10.0,"MIN":0.1,"MAX":30.0},

    {"NAME":"brightness","TYPE":"float","DEFAULT":1.0,"MIN":0.0,"MAX":2.0},
    {"NAME":"saturation","TYPE":"float","DEFAULT":1.0,"MIN":0.0,"MAX":2.0},
    {"NAME":"contrast","TYPE":"float","DEFAULT":1.0,"MIN":0.0,"MAX":3.0},
    {"NAME":"vignetteStrength","TYPE":"float","DEFAULT":0.5,"MIN":0.0,"MAX":2.0},
    {"NAME":"expFactor","TYPE":"float","DEFAULT":6.0,"MIN":1.0,"MAX":20.0},

    {"NAME":"EnableLiminalWarp","TYPE":"bool","DEFAULT":false},
    {"NAME":"LiminalMix","TYPE":"float","DEFAULT":0.35,"MIN":0.0,"MAX":1.0},
    {"NAME":"LiminalScale","TYPE":"float","DEFAULT":1.0,"MIN":0.1,"MAX":5.0},

    {"NAME":"fractalMorph","TYPE":"float","DEFAULT":0.5,"MIN":0.0,"MAX":1.0},
    {"NAME":"TransformMode","TYPE":"float","DEFAULT":1.8,"MIN":0.0,"MAX":5.0},
    {"NAME":"GeometryType","TYPE":"float","DEFAULT":3.0,"MIN":0.0,"MAX":6.0},

    {"NAME":"ChaosIntensity","TYPE":"float","DEFAULT":0.43,"MIN":0.0,"MAX":2.0},
    {"NAME":"ChaosSpeed","TYPE":"float","DEFAULT":0.66,"MIN":0.1,"MAX":4.0},

    {"NAME":"Texture","TYPE":"image"},
    {"NAME":"TextureWarp","TYPE":"float","DEFAULT":0.25,"MIN":0.0,"MAX":2.0},
    {"NAME":"TextureScale","TYPE":"float","DEFAULT":1.0,"MIN":0.1,"MAX":10.0},

    {"NAME":"backgroundColor","TYPE":"color","DEFAULT":[0.0,0.0,0.0,1.0]}
  ]
}*/

#define PI 3.14159265359
#define TAU 6.28318530718

float iTime;
vec3 dynamicPalette[7];

// ---------------- math helpers ----------------
mat2 rot2D(float a){
    float c = cos(a), s = sin(a);
    return mat2(c, -s, s, c);
}

vec3 pal(float t, vec3 a, vec3 b, vec3 c, vec3 d){
    return a + b * cos(TAU*(c*t + d));
}

vec3 getColorPalette(float mode, float t){
    if (mode < 8.0) {
        float id = floor(mode);
        if (id < 1.0) return 0.5 + 0.5 * cos(TAU*(t + vec3(0.0,0.33,0.67)));
        if (id < 2.0) return vec3(sin(t*3.0), sin(t*2.5+1.0), sin(t*4.0+2.0)) * 1.1;
        if (id < 3.0) return vec3(1.0 - abs(sin(t*3.0 + vec3(0.5,0.3,0.1)))) * 1.2;
        if (id < 4.0) return vec3(0.3 + 0.4 * sin(t + vec3(1.0,2.0,3.0)));
        if (id < 5.0) return vec3(sin(t*7.0), sin(t*13.0), sin(t*17.0));
        if (id < 6.0) return vec3(1.0, 0.7 + 0.3 * sin(t*3.5), 0.6 * sin(t*2.0));
        return vec3(exp(-t*2.0)) * vec3(1.2,0.8,1.5);
    }
    return pal(t,
        vec3(0.5 + 0.4*sin(mode*0.5), 0.6 + 0.3*cos(mode*1.2), 0.4 + 0.5*sin(mode*0.9)),
        vec3(0.4),
        vec3(1.0,1.3,0.7),
        vec3(0.1,0.2,0.3)
    );
}

// ---------------- camera path ----------------
vec3 P(float z){
    return vec3(
        tanh(cos(z * P_X_CosFactor) * P_X_Amp) * P_X_Range,
        tanh(cos(z * P_Y_CosFactor) * P_Y_Amp) * P_Y_Range,
        z
    );
}

// ---------------- triplanar ----------------
vec3 triplanarTexture(vec3 p, float scale){
    vec3 n = normalize(abs(p) + 1e-6);
    vec3 blend = pow(n, vec3(4.0));
    blend /= dot(blend, vec3(1.0));
    vec2 xz = fract(p.zy * scale);
    vec2 yz = fract(p.xz * scale);
    vec2 xy = fract(p.xy * scale);
    vec3 tx = IMG_NORM_PIXEL(Texture, xz).rgb;
    vec3 ty = IMG_NORM_PIXEL(Texture, yz).rgb;
    vec3 tz = IMG_NORM_PIXEL(Texture, xy).rgb;
    return tx * blend.x + ty * blend.y + tz * blend.z;
}

// ---------------- fractal fields ----------------
float fractalField(vec3 p){
    float w = 0.1;
    vec3 q = p;
    int ITER = int(clamp(fractalIterations,1.0,20.0));
    for(int j=0;j<ITER;j++){
        q = abs(sin(q)) - fractalOffset;
        float l = fractalAmp / max(dot(q,q),1e-6);
        q *= l; w *= l;
        if(length(q) > 1e6) break;
    }
    return length(q) / max(abs(w),1e-6);
}

float liminalFractal(vec3 p, float t, float scale){
    vec3 q = p * scale;
    float m = 1e6;
    for(int i=0;i<6;i++){
        q = abs(q)/max(dot(q,q),1e-6) - fractalMorph;
        q.xy *= rot2D(t*0.05+float(i)*0.1);
        m = min(m,length(q));
    }
    return m;
}

// ---------------- geometry SDFs ----------------
float sdSphere(vec3 p,float r){ return length(p)-r; }
float sdTorus(vec3 p,vec2 t){ vec2 q=vec2(length(p.xz)-t.x,p.y); return length(q)-t.y; }
float sdGrid(vec3 p,float freq){ return sin(p.x*freq)*sin(p.y*freq)*sin(p.z*freq); }

float shapeSpikeFractal(vec3 p){
    float d=0.0; int K=int(clamp(tunnelRaymarchSteps,1.0,128.0));
    for(int i=0;i<K;i++){
        p=abs(p)/max(dot(p,p),1e-6)-0.5;
        p*=0.95;
        d+=length(p);
    }
    return d*0.05;
}

float geometryBase(vec3 p){
    float g = GeometryType;
    if(g<0.5) return sdSphere(p,1.0);
    else if(g<1.5) return sdTorus(p,vec2(1.0,0.3));
    else if(g<2.5) return shapeSpikeFractal(p);
    else if(g<3.5) return sdGrid(p,4.0);
    else{
        float lim = liminalFractal(p,iTime,LiminalScale);
        float spike = shapeSpikeFractal(p);
        float mixAmt = EnableLiminalWarp ? clamp(LiminalMix,0.0,1.0) : 0.0;
        return mix(spike, lim, mixAmt);
    }
}

// ---------------- chaos / transform (constrained) ----------------
vec3 applyTransformModes(vec3 p,float mode){
    float chspd = ChaosSpeed;
    float chaos = ChaosIntensity;

    vec3 orig = p;
    if(mode<1.5){
        p = abs(p);
    } else if(mode<2.5){
        p += sin(p*3.0 + iTime*chspd) * (chaos*0.30);
    } else if(mode<3.5){
        p += sin(p*(1.0+chaos*2.0) + iTime*chspd) * (chaos*0.50);
        p = fract(p*1.5) - 0.75;
    } else if(mode<4.5){
        float a = atan(p.z,p.x);
        float r = max(0.0001, length(p.xz));
        a += iTime*chspd*0.2;
        p.x = cos(a)*r; p.z = sin(a)*r;
    } else {
        p.xz = rot2D(iTime*chspd*0.3) * p.xz;
    }

    // Keep within tunnel using soft clamp towards original axis
    // Compute where the tunnel center is at this z:
    vec2 center = P(p.z).xy;
    float dynR = 1.5 * (tunnelDensityOffset + tunnelDensityFactor); // rough radius
    float distFromCenter = length((p.xy) - center);

    // When outside dynR, pull back toward center
    float pull = smoothstep(dynR, dynR*1.6, distFromCenter);
    p.xy = mix(p.xy, center + normalize(p.xy-center)*dynR, pull);

    // Also limit absolute radius in XY to avoid exploding
    float hardR = dynR*2.0 + 10.0;
    if(length(p.xy-center) > hardR){
        p.xy = center + normalize(p.xy-center)*hardR;
    }

    // Blend a little with original to avoid sudden jumps
    p = mix(orig, p, clamp(0.25 + chaos*0.35, 0.0, 1.0));
    return p;
}

// ---------------- color utils ----------------
vec3 applyShimmer(vec3 col,vec2 uv,float t){
    if(shimmerStrength>0.001){
        float s = fract(sin(uv.x*50.0+t*shimmerSpeed)+cos(uv.y*70.0+t*shimmerSpeed*1.3));
        vec3 sc = getColorPalette(paletteSelect+0.5, s + t*0.01) * paletteBrightness;
        col += sc * (shimmerStrength*0.12);
    }
    return col;
}

vec3 applyColorAdjustments(vec3 col){
    float luma = dot(col, vec3(0.2126,0.7152,0.0722));
    col = mix(vec3(luma), col, saturation);
    col *= contrast;
    col *= brightness;

    // subtle vignette using exp falloff
    vec2 uv = (gl_FragCoord.xy - RENDERSIZE.xy*0.5)/RENDERSIZE.xy;
    float v = exp(-dot(uv,uv)*expFactor)*vignetteStrength;
    col *= mix(1.0, v, clamp(vignetteStrength*0.5, 0.0, 1.0));
    return col;
}

// ---------------- MAIN ----------------
void main(){
    iTime = TIME * masterSpeed * 2.5;

    // always start with background color, will add light over it
    vec3 outColor = backgroundColor.rgb;
    vec3 accum     = vec3(0.0);

    // build dynamic palette
    for(int j=0;j<7;j++){
        dynamicPalette[j] = getColorPalette(paletteSelect + float(j)*0.14, iTime*(1.0+paletteAnimSpeed)) * paletteBrightness;
    }

    // UV + shake
    vec2 frag = gl_FragCoord.xy;
    if(shakeAmount>0.001){
        float sx = sin(iTime*shakeSpeed*15.0)*cos(iTime*shakeSpeed*10.0)*shakeAmount*RENDERSIZE.y;
        float sy = cos(iTime*shakeSpeed*12.0)*sin(iTime*shakeSpeed*18.0)*shakeAmount*RENDERSIZE.y;
        frag += vec2(sx,sy);
    }
    vec2 uv = (frag - RENDERSIZE.xy*0.5)/RENDERSIZE.y;

    // camera setup
    vec3 ro, ta;
    if(enableAutoCameraMovement){ ro=P(iTime*autoCamSpeed); ta=P(iTime*autoCamSpeed+1.0); }
    else { ro=vec3(camX,camY,camZ); ta=vec3(lookAtX,lookAtY,lookAtZ); }

    vec3 Z = normalize(ta-ro);
    vec3 up = abs(dot(Z,vec3(0.0,1.0,0.0)))>0.999 ? vec3(0.0,0.0,1.0) : vec3(0.0,1.0,0.0);
    vec3 X = normalize(cross(up,Z));
    vec3 Y = normalize(cross(Z,X));

    float rot1 = sin(iTime*cameraRotSpeed)*cameraRotAmount;
    float rot2 = tanh(sin(ro.z*0.1)*cameraRotZOffsetSpeed)*cameraRotZOffsetAmount;
    mat2 R = rot2D(rot1) * rot2D(rot2);

    vec3 D  = normalize(vec3(R*uv, camFOV));
    mat3 basis = mat3(-X, cross(X,Z), Z);
    D = normalize(basis*D);

    // -------- tunnel march (hit surface where signed distance ~ 0) --------
    float d_total = 0.0;
    float s_dist  = raymarchMinDist;
    bool tunnelHit = false;

    int steps_t = int(clamp(tunnelRaymarchSteps,8.0,500.0));
    for(int i=0;i<steps_t;i++){
        vec3 pos = ro + D*d_total;

        // apply (constrained) chaos/transforms while marching tunnel
        pos = applyTransformModes(pos, TransformMode);

        float dynDensity = (tunnelDensityOffset + tanh(sin(iTime*tunnelDensitySpeed)*0.5)*tunnelDensityFactor) * 1.5;
        float sdfTunnel  = dynDensity - length(pos.xy - P(pos.z).xy);

        if(sdfTunnel < raymarchMinDist){ tunnelHit = true; break; }
        if(d_total > raymarchMaxDist) break;

        s_dist = max(raymarchMinDist, abs(sdfTunnel));
        d_total += s_dist;
    }

    // -------- fractal/geo march inside tunnel --------
    vec3 ro_f = ro + D*d_total;
    float d_f = 0.0;
    bool fractalHit = false;

    int steps_f = int(clamp(fractalRaymarchSteps,8.0,500.0));
    for(int i=0;i<steps_f;i++){
        vec3 p = ro_f + D*d_f;

        // lock to tunnel centerline frame
        p.xy -= P(p.z).xy;

        // apply (constrained) chaos/transforms
        p = applyTransformModes(p, TransformMode);

        float geomDist = geometryBase(p);
        float fdist    = fractalField(p);
        float dist     = mix(geomDist, fdist, 0.5);

        // accumulate a soft glow based on proximity
        float glow = fractalGlowStrength * exp(-abs(dist)*8.0);
        accum += glow * globalGlowIntensity;

        float stepSize = max(0.0005, abs(dist));
        if(stepSize < raymarchMinDist){ fractalHit = true; break; }
        if(d_f > raymarchMaxDist) break;
        d_f += stepSize;
    }

    // -------- shading / color --------
    vec3 finalPos = ro_f + D*d_f;

    // palette-based modulation
    float palCoord = finalPos.z*0.1 + iTime*0.01;
    vec3 basePal   = getColorPalette(paletteSelect, palCoord) * paletteBrightness;

    // if we hit something, color = background + accumulated light, tinted by palette
    if(tunnelHit || fractalHit){
        vec3 color = accum;

        // add a subtle pattern from world pos to avoid flat look
        color *= (basePal + 0.5 + 0.5*sin(finalPos));

        // optional texture warp
        if(TextureWarp>0.001 && TextureScale>0.0){
            vec3 tx = triplanarTexture(finalPos*TextureScale, 1.0);
            color = mix(color, tx, clamp(TextureWarp,0.0,1.0)*0.35);
        }

        // pulse lines (optional visual on top)
        if(pulseLineIntensity > 0.0){
            float dir = floor(clamp(pulseLineDirection,0.0,3.0)+0.5);
            float coord = (dir<1.5) ? finalPos.z : (dir<2.5 ? finalPos.x : finalPos.y);
            float stripe = smoothstep(0.0, pulseLineThickness, abs(fract(coord*pulseLineSpeed) - 0.5));
            color += (1.0 - stripe) * pulseLineIntensity * basePal;
        }

        color = applyShimmer(color, uv, iTime);
        color = applyColorAdjustments(color);

        // ALWAYS blend with background, not replace
        // simple additive with soft rolloff to avoid blowing out
        color = 1.0 - exp(-color);               // soft tonemap of emission
        outColor = mix(outColor, color, 0.9);    // blend over background
    }

    gl_FragColor = vec4(outColor, 1.0);
}
