/*
{
	"CATEGORIES": [
		"Fractal",
		"Psychedelic",
		"3D",
		"Animated",
		"Abstract",
		"Volumetric",
		"Optimized"
	],
	"DESCRIPTION": "A complete hybrid shader merging all features from two distinct fractal shaders. The core psychedelic folding logic is injected into a volumetric framework, with new sliders to control the dominance of each component's geometry and color. Includes all camera controls, glitches, and post-processing from both original shaders.",
	"CREDIT": "Completely merged and debugged by Gemini, based on two original user-provided shaders.",
	"ISF_VERSION": "2.0",
	"INPUTS": [
		{ "NAME": "speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 5.0, "DESCRIPTION": "Overall animation speed." },
		{ "NAME": "zoom", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.1, "MAX": 5.0, "DESCRIPTION": "Camera zoom level. Higher values zoom in." },
		{ "NAME": "fractal_dominance", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Mix factor between the core psychedelic fractal logic and the secondary chaos logic. 0.0 = only secondary logic, 1.0 = only core fractal logic." },
		{ "NAME": "texture_dominance", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Mix factor between the psychedelic color palette and the triplanar texture color. 0.0 = only palette, 1.0 = only texture." },
		{ "NAME": "CameraOrbit", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "DESCRIPTION": "Camera orbit rotation." },
		{ "NAME": "CameraPitch", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.57, "MAX": 1.57, "DESCRIPTION": "Camera pitch rotation." },
		{ "NAME": "CameraRoll", "TYPE": "float", "DEFAULT": 0.0, "MIN": -3.14, "MAX": 3.14, "DESCRIPTION": "Camera roll rotation." },
		{ "NAME": "morph_factor", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Influences various aspects of fractal morphing (e.g., offsets, folding)." },
		{ "NAME": "raymarch_iterations", "TYPE": "float", "DEFAULT": 90.0, "MIN": 10.0, "MAX": 200.0, "STEP": 1.0, "DESCRIPTION": "Number of raymarching steps. Higher values increase detail but reduce performance." },
		{ "NAME": "fractal_inner_iterations", "TYPE": "float", "DEFAULT": 8.0, "MIN": 1.0, "MAX": 15.0, "STEP": 1.0, "DESCRIPTION": "Number of iterations for the inner fractal folding loop." },
		{ "NAME": "base_rotation_speed_1", "TYPE": "float", "DEFAULT": 0.03, "MIN": 0.0, "MAX": 0.5, "DESCRIPTION": "Speed for the first base rotation of the fractal." },
		{ "NAME": "base_rotation_speed_2", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.0, "MAX": 0.5, "DESCRIPTION": "Speed for the second base rotation of the fractal." },
		{ "NAME": "cosine_fold_mult_1", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 10.0, "DESCRIPTION": "Multiplier for the first cosine fold term." },
		{ "NAME": "cosine_fold_mult_2", "TYPE": "float", "DEFAULT": 4.0, "MIN": 0.0, "MAX": 10.0, "DESCRIPTION": "Multiplier for the second cosine fold term." },
		{ "NAME": "cosine_fold_mult_3", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Multiplier for the nested cosine fold term." },
		{ "NAME": "q_sin_time_mult", "TYPE": "float", "DEFAULT": 0.2, "MIN": -1.0, "MAX": 1.0, "DESCRIPTION": "Multiplier for sin(TIME) added to Q vector (morphs fractal)." },
		{ "NAME": "clamp_limit", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Clamping limit for the fractal folding operation." },
		{ "NAME": "inner_scale_factor", "TYPE": "float", "DEFAULT": 7.0, "MIN": 1.0, "MAX": 20.0, "DESCRIPTION": "Scaling factor for the inner fractal iterations." },
		{ "NAME": "inner_scale_clamp_val", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Clamp value used in inner scale calculation." },
		{ "NAME": "color_palette_type", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 6.0, "STEP": 1.0, "DESCRIPTION": "Selects one of 7 psychedelic color palettes." },
		{ "NAME": "color_pulse_speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 5.0, "DESCRIPTION": "Speed of color pulsing." },
		{ "NAME": "color_pulse_intensity", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Intensity of color pulsing." },
		{ "NAME": "log_scale_color_factor", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Factor for log(s) in hue calculation." },
		{ "NAME": "base_color_mix", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Mix factor between white and fractal color (0=white, 1=fractal color)." },
		{ "NAME": "base_color_multiplier", "TYPE": "float", "DEFAULT": 0.03, "MIN": 0.001, "MAX": 0.1, "DESCRIPTION": "Base intensity multiplier for accumulated color." },
		{ "NAME": "exponent_intensity", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 5.0, "DESCRIPTION": "Intensity of the exponent term in color calculation." },
		{ "NAME": "final_color_power", "TYPE": "float", "DEFAULT": 4.0, "MIN": 1.0, "MAX": 10.0, "DESCRIPTION": "Power applied to final color for contrast." },
		{ "NAME": "glitch_strength", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Strength of glitch effect (0=none, 1=max)." },
		{ "NAME": "glitch_frequency", "TYPE": "float", "DEFAULT": 10.0, "MIN": 0.1, "MAX": 50.0, "DESCRIPTION": "Frequency of glitch disruptions." },
		{ "NAME": "shake_strength", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Strength of camera shake effect (0=none, 1=max)." },
		{ "NAME": "shake_frequency", "TYPE": "float", "DEFAULT": 20.0, "MIN": 0.1, "MAX": 50.0, "DESCRIPTION": "Frequency of camera shake oscillations." },
		{ "NAME": "brightness", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Adjusts overall brightness." },
		{ "NAME": "saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Adjusts overall saturation." },
		{ "NAME": "contrast", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Adjusts overall contrast." },
		{ "NAME": "Glow", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Adds a glow effect to the scene." },
		{ "NAME": "Symmetry", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 4.0, "DESCRIPTION": "Controls the symmetry of the fractal transform." },
		{ "NAME": "ChaosIntensity", "TYPE": "float", "DEFAULT": 0.43, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Intensity of the chaos-based geometry distortion." },
		{ "NAME": "ChaosSpeed", "TYPE": "float", "DEFAULT": 0.66, "MIN": 0.1, "MAX": 4.0, "DESCRIPTION": "Speed of the chaos animation." },
		{ "NAME": "Texture", "TYPE": "image", "DESCRIPTION": "Texture to be used for the scene color." },
		{ "NAME": "TextureWarp", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0, "DESCRIPTION": "Warp the camera position using the texture." },
		{ "NAME": "TextureScale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0, "DESCRIPTION": "Scale factor for the triplanar texture mapping." },
		{ "NAME": "FalloffCurve", "TYPE": "float", "DEFAULT": 1.1, "MIN": 0.1, "MAX": 3.0, "DESCRIPTION": "Controls the falloff of color with distance." },
		{ "NAME": "TransformMode", "TYPE": "float", "DEFAULT": 1.8, "MIN": 0, "MAX": 5, "STEP": 1.0, "DESCRIPTION": "Selects the geometric transform mode." },
		{ "NAME": "GeometryType", "TYPE": "float", "DEFAULT": 3, "MIN": 0, "MAX": 6, "STEP": 1.0, "DESCRIPTION": "Selects the base fractal geometry type." },
		{ "NAME": "ChaosMix", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Mix factor between the base geometry and the chaotic sine wave distortion." },
		{ "NAME": "Sharpen", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0, "DESCRIPTION": "Adjusts the final sharpness of the image." }
	]
}
*/

#define PI 3.14159
#define BAILOUT 100.0

// Rotation utility from first shader
#define R(p,a,r) mix(a*dot(p,a),p,cos(r))+sin(r)*cross(p,a)
// Hue-based color utility from first shader
#define H(h)(cos((h)*6.3+vec3(0,23,21))*.5+.5)

// Camera matrix from second shader
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

// Compact color palette function from first shader
vec3 getColorPalette(float t, float type) {
	if (type < 0.5) return H(t); // Palette 0: Default Psychedelic Flow
	if (type < 1.5) return vec3(sin(t * 5.0), sin(t * 7.0 + 1.0), sin(t * 9.0 + 2.0)) * 0.5 + 0.5; // Palette 1: Rapid Sine Waves
	if (type < 2.5) return vec3(cos(t * 4.0 + 2.0), cos(t * 2.0 + 1.0), sin(t * 6.0)) * 0.5 + 0.5; // Palette 2: Muted Cosine Blends
	if (type < 3.5) return vec3(sin(t * 2.0), sin(t * 4.0), cos(t * 8.0)) * 0.5 + 0.5; // Palette 3: Fast RGB Pulse
	if (type < 4.5) return vec3(fract(t * 3.0), fract(t * 5.0), fract(t * 7.0)); // Palette 4: Hard Edge Fractal Colors
	if (type < 5.5) return vec3(sin(t * 1.5), cos(t * 3.0), sin(t * 4.5 + cos(t * 2.0))) * 0.5 + 0.5; // Palette 5: Organic Swirl
	return mix(H(t * 0.7), H(t * 1.3 + 0.5), 0.5); // Palette 6: Dual Hue Blend
}

// Color adjustment function from first shader
vec3 adjustColor(vec3 color, float br, float sat, float con) {
	color = mix(vec3(0.5), color, con);
	vec3 gray = vec3(dot(color, vec3(0.299, 0.587, 0.114)));
	color = mix(gray, color, sat);
	return color * br;
}

// Simple hash functions for noise for glitch/shake from first shader
float hash11(float p) { p = fract(p * .1031); p *= p + 33.33; p *= p + p; return fract(p); }
float hash22(vec2 p) { return fract(sin(dot(p, vec2(41.45, 12.04))) * 9876.5432); }

// Triplanar texture mapping from second shader
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

// Secondary geometry distortion function from second shader
float shapeChaos(vec3 p) {
	return (sin(p.x*3. + TIME*ChaosSpeed) + sin(p.y*4. + TIME*ChaosSpeed*1.2) + sin(p.z*5. + TIME*ChaosSpeed*0.8)) * ChaosIntensity;
}

// Spike fractal from second shader
float shapeSpikeFractal(vec3 p) {
   float d = 0.0;
   for (int i = 0; i < int(raymarch_iterations); i++) {
		p = abs(p) / dot(p, p + 0.001) - 0.5;
		p *= 0.95;
		d += length(p);
   }
   return d / 20.0;
}

// Core fractal distance field function from first shader
float fractalDistance(vec3 p) {
	// Apply base rotation for fractal
	vec3 fractal_offset_base = vec3(1.0, 0.8, 1.2);
	vec3 fractal_offset = mix(fractal_offset_base, fractal_offset_base * (1.0 + morph_factor * 0.5), morph_factor);

	p += R(fractal_offset, vec3(0.577), TIME * speed * base_rotation_speed_1);
	p = R(p, vec3(0.577), TIME * speed * base_rotation_speed_2);

	// Core Fractal Folding Logic
	p = cos(p * cosine_fold_mult_1 + cosine_fold_mult_2 * cos(p * cosine_fold_mult_3));
	
	vec4 q = vec4(p, sin(TIME * speed) * q_sin_time_mult);
	float innerScale = 3.0;

	// Inner loop for self-similarity
	for(int k = 0; k < int(fractal_inner_iterations); k++) {
		q = clamp(q, -clamp_limit, clamp_limit) * 2.0 - q;
		float distanceEstimate = inner_scale_factor * clamp(inner_scale_clamp_val / max(dot(q.xyz, q.xyz), 0.000001), 0.0, 1.0);
		innerScale *= distanceEstimate;
		q *= distanceEstimate;
	}
	
	// Distance accumulation
	return length(q) / innerScale;
}

// Base scene function from second shader, using GeometryType
float scene(vec3 p, float geo) {
	float base;
	if (geo < 0.5) base = length(p) - 1.0;
	else if (geo < 1.5) {
		vec2 q = vec2(length(p.xz) - 1.0, p.y);
		base = length(q) - 0.3;
	}
	else if (geo < 2.5) base = fractalDistance(p); // Use core fractal as a geometry option
	else if (geo < 3.5) base = shapeSpikeFractal(p * 1.2);
	else base = fractalDistance(p); // Default or other modes
	
	return mix(base, shapeChaos(p), ChaosMix);
}

// Function to apply transforms from the second shader's logic
vec3 applyTransform(vec3 p, float mode) {
	p *= max(Symmetry, 0.001);
	if (mode < 0.5) p = p; // No transform
	else if (mode < 1.5) p = abs(p);
	else if (mode < 2.5) p += sin(p * 3.0 + TIME * ChaosSpeed) * ChaosIntensity * 0.3;
	else if (mode < 3.5) {
		p += sin(p * (1.0 + ChaosIntensity * 2.0) + TIME * ChaosSpeed) * ChaosIntensity * 0.5;
		p = fract(p * 1.5) - 0.75;
	}
	else {
		float a = atan(p.z, p.x);
		float r = length(p.xz);
		float spin = TIME * ChaosSpeed * (mode < 4.5 ? 0.2 : 0.3);
		a += spin;
		p.x = cos(a) * r;
		p.z = sin(a) * r;
	}
	return p;
}


void main() {
	vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE.xy) / RENDERSIZE.y;
	
	// Apply glitch and shake from first shader
	float current_time = TIME * speed;
	if (shake_strength > 0.001) {
		vec2 shake_offset = vec2(
			sin(current_time * shake_frequency + hash11(1.0)) * 0.1,
			cos(current_time * shake_frequency * 1.1 + hash11(2.0)) * 0.1
		) * shake_strength;
		uv += shake_offset;
	}
	if (glitch_strength > 0.001) {
		float offset_x = (hash22(uv * 10.0 + current_time * glitch_frequency) - 0.5) * 2.0;
		float scanline_strength = sin(uv.y * 150.0 + current_time * 20.0) * 0.5 + 0.5;
		uv.x += offset_x * scanline_strength * glitch_strength * 0.05;
	}

	uv *= zoom;

	vec3 ro = vec3(0.0, 0.0, -3.0);
	vec3 rd = normalize(vec3(uv, 1.0));
	rd = cameraMatrix(CameraOrbit, CameraPitch, CameraRoll) * rd;
	
	// Apply texture warp to camera position from second shader
	vec3 warp = triplanarTexture(ro * TextureScale, 1.0) - 0.5;
	vec3 roWarped = ro + warp * TextureWarp;

	vec3 accumulatedColor = vec3(0.0);
	float dist = 0.0;

	for (int i = 0; i < int(raymarch_iterations); i++) {
		vec3 p = roWarped + dist * rd;
		
		// Apply transforms from second shader
		p = applyTransform(p, TransformMode);
		
		// Calculate two distance estimates to blend them.
		// The `scene` function provides the base geometry, which is then blended with the chaos logic.
		float d_base_geometry = scene(p, GeometryType);
		float d_fractal_core = fractalDistance(p);
		
		// Mix the geometry logic based on the `fractal_dominance` slider
		float d = mix(d_base_geometry, d_fractal_core, fractal_dominance);
		
		d = max(abs(d), 0.01);
		
		// FalloffCurve is now correctly applied here
		float fading_factor = pow(exp(-float(i) * d * exponent_intensity * 0.0001), FalloffCurve);
		float pulse_val = sin(float(i) * 0.1 + current_time * color_pulse_speed) * color_pulse_intensity;
		float log_val = log(max(0.001, d)) * log_scale_color_factor;

		// Calculate colors from both shaders
		vec3 palCol = getColorPalette(log_val + pulse_val, color_palette_type);
		vec3 texCol = triplanarTexture(p * TextureScale, 1.0);

		// Blend the two color sources using the new `texture_dominance` slider
		vec3 final_color_at_step = mix(palCol, texCol, texture_dominance);
		
		// Apply color based on distance and fading
		accumulatedColor += mix(vec3(1.0), final_color_at_step, base_color_mix) * base_color_multiplier * fading_factor;
		
		dist += d;
		if (dist > BAILOUT || d < 0.0001) break;
	}

	// Final post-processing, combining effects from both shaders
	accumulatedColor = pow(accumulatedColor, vec3(final_color_power));
	accumulatedColor = adjustColor(accumulatedColor, brightness, saturation, contrast);
	
	// Add glow effect from second shader
	accumulatedColor *= Glow;
	
	// Vignette effect from first shader
	vec2 vignette_uv = gl_FragCoord.xy / RENDERSIZE.xy;
	float vignette_val = pow(16.0 * vignette_uv.x * vignette_uv.y * (1.0 - vignette_uv.x) * (1.0 - vignette_uv.y), 0.3);
	accumulatedColor *= mix(1.0, vignette_val, 0.5);

	// Sharpen effect is added here
	accumulatedColor = mix(accumulatedColor, pow(accumulatedColor, vec3(1.0 + Sharpen)), Sharpen);

	gl_FragColor = vec4(clamp(accumulatedColor, 0.0, 1.0), 1.0);
}
