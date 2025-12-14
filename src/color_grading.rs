//! Professional Color Grading System for WGSL Shader Studio
//!
//! Provides comprehensive color grading tools including curves, levels, LUT support,
//! and real-time color correction for professional video production.

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Color grading control point for curves
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
    pub locked: bool,
}

impl CurvePoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, locked: false }
    }
    
    pub fn locked(x: f32, y: f32) -> Self {
        Self { x, y, locked: true }
    }
}

/// Curve type for color grading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurveType {
    Master,     // RGB combined
    Red,        // Red channel only
    Green,      // Green channel only
    Blue,       // Blue channel only
    RGB,        // Individual RGB curves
    Luma,       // Luminance curve
    Saturation, // Saturation vs luminance curve
}

/// Color grading curve with control points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorCurve {
    pub curve_type: CurveType,
    pub points: Vec<CurvePoint>,
    pub interpolation: CurveInterpolation,
    pub enabled: bool,
}

/// Curve interpolation method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurveInterpolation {
    Linear,
    Cubic,
    CatmullRom,
    Bezier,
}

impl Default for ColorCurve {
    fn default() -> Self {
        Self {
            curve_type: CurveType::Master,
            points: vec![
                CurvePoint::locked(0.0, 0.0),
                CurvePoint::new(0.5, 0.5),
                CurvePoint::locked(1.0, 1.0),
            ],
            interpolation: CurveInterpolation::Cubic,
            enabled: true,
        }
    }
}

/// Levels adjustment parameters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LevelsAdjustment {
    pub input_black: f32,
    pub input_white: f32,
    pub gamma: f32,
    pub output_black: f32,
    pub output_white: f32,
}

impl Default for LevelsAdjustment {
    fn default() -> Self {
        Self {
            input_black: 0.0,
            input_white: 1.0,
            gamma: 1.0,
            output_black: 0.0,
            output_white: 1.0,
        }
    }
}

/// Color wheels for primary/secondary color correction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorWheel {
    pub shadows: Vec2,      // Shadow color balance
    pub midtones: Vec2,     // Midtone color balance
    pub highlights: Vec2,   // Highlight color balance
}

impl Default for ColorWheel {
    fn default() -> Self {
        Self {
            shadows: Vec2::ZERO,
            midtones: Vec2::ZERO,
            highlights: Vec2::ZERO,
        }
    }
}

/// LUT (Look-Up Table) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LutInfo {
    pub name: String,
    pub file_path: String,
    pub size: u32, // Usually 16, 32, or 64
    pub format: LutFormat,
    pub data: Vec<f32>, // RGB values
}

/// LUT file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LutFormat {
    Cube,       // .cube format
    ThreeDl,    // .3dl format
    Csp,        // .csp format
    Icc,        // .icc profile
    Image,      // 2D/3D texture representation
}

/// Basic color correction parameters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BasicColorCorrection {
    pub exposure: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub whites: f32,
    pub blacks: f32,
    pub vibrance: f32,
    pub saturation: f32,
    pub temperature: f32, // Kelvin temperature
    pub tint: f32,        // Green-magenta tint
}

impl Default for BasicColorCorrection {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            vibrance: 0.0,
            saturation: 0.0,
            temperature: 6500.0,
            tint: 0.0,
        }
    }
}

/// Histogram data for real-time analysis
#[derive(Debug, Clone)]
pub struct HistogramData {
    pub rgb: [Vec<u32>; 3], // R, G, B histograms (256 bins)
    pub luma: Vec<u32>,     // Luminance histogram (256 bins)
    pub rgb_combined: Vec<u32>, // Combined RGB histogram (256 bins)
    pub waveform: Vec<Vec<u32>>, // 2D waveform data
    pub vectorscope: Vec<Vec<u32>>, // 2D vectorscope data
}

impl Default for HistogramData {
    fn default() -> Self {
        Self {
            rgb: [vec![0; 256], vec![0; 256], vec![0; 256]],
            luma: vec![0; 256],
            rgb_combined: vec![0; 256],
            waveform: vec![vec![0; 256]; 256],
            vectorscope: vec![vec![0; 256]; 256],
        }
    }
}

/// Main color grading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGradingConfig {
    pub enabled: bool,
    pub curves: HashMap<String, ColorCurve>,
    pub levels: LevelsAdjustment,
    pub color_wheels: ColorWheel,
    pub basic_correction: BasicColorCorrection,
    pub luts: Vec<LutInfo>,
    pub active_lut: Option<String>,
}

impl Default for ColorGradingConfig {
    fn default() -> Self {
        let mut curves = HashMap::new();
        curves.insert("master".to_string(), ColorCurve::default());
        curves.insert("red".to_string(), ColorCurve {
            curve_type: CurveType::Red,
            ..Default::default()
        });
        curves.insert("green".to_string(), ColorCurve {
            curve_type: CurveType::Green,
            ..Default::default()
        });
        curves.insert("blue".to_string(), ColorCurve {
            curve_type: CurveType::Blue,
            ..Default::default()
        });

        Self {
            enabled: true,
            curves,
            levels: LevelsAdjustment::default(),
            color_wheels: ColorWheel::default(),
            basic_correction: BasicColorCorrection::default(),
            luts: Vec::new(),
            active_lut: None,
        }
    }
}

/// Color grading system resource
#[derive(Resource)]
pub struct ColorGradingSystem {
    pub config: ColorGradingConfig,
    pub histogram: HistogramData,
    pub enabled: bool,
    pub real_time_analysis: bool,
}

impl Default for ColorGradingSystem {
    fn default() -> Self {
        Self {
            config: ColorGradingConfig::default(),
            histogram: HistogramData::default(),
            enabled: false,
            real_time_analysis: true,
        }
    }
}

impl ColorGradingSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply color grading to RGB values
    pub fn apply_grading(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        if !self.enabled {
            return (r, g, b);
        }

        let (mut r, mut g, mut b) = (r, g, b);

        // Apply basic color correction first
        (r, g, b) = self.apply_basic_correction(r, g, b);

        // Apply curves
        (r, g, b) = self.apply_curves(r, g, b);

        // Apply levels
        (r, g, b) = self.apply_levels(r, g, b);

        // Apply color wheels
        (r, g, b) = self.apply_color_wheels(r, g, b);

        // Apply LUT if active
        if let Some(lut_name) = &self.config.active_lut {
            (r, g, b) = self.apply_lut(r, g, b, lut_name);
        }

        // Clamp values
        (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }

    /// Apply basic color correction
    fn apply_basic_correction(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let correction = &self.config.basic_correction;
        
        // Exposure (in stops)
        let exposure_factor = 2.0f32.powf(correction.exposure);
        let (mut r, mut g, mut b) = (r * exposure_factor, g * exposure_factor, b * exposure_factor);

        // Contrast (centered at 0.5)
        let contrast_factor = (correction.contrast + 100.0) / 100.0;
        r = (r - 0.5) * contrast_factor + 0.5;
        g = (g - 0.5) * contrast_factor + 0.5;
        b = (b - 0.5) * contrast_factor + 0.5;

        // Temperature and tint
        let temp_factor = (correction.temperature - 6500.0) / 6500.0;
        let tint_factor = correction.tint / 100.0;
        
        // Simple temperature/tint adjustment
        r += temp_factor * 0.1 + tint_factor * 0.05;
        g += temp_factor * -0.05;
        b += temp_factor * -0.1 - tint_factor * 0.05;

        // Vibrance (smart saturation)
        let avg = (r + g + b) / 3.0;
        let saturation = correction.saturation / 100.0;
        let vibrance = correction.vibrance / 100.0;
        
        // Apply vibrance to less saturated colors more
        let saturation_amount = saturation + vibrance * (1.0 - ((r - avg).abs() + (g - avg).abs() + (b - avg).abs()) / 3.0);
        
        r = avg + (r - avg) * (1.0 + saturation_amount);
        g = avg + (g - avg) * (1.0 + saturation_amount);
        b = avg + (b - avg) * (1.0 + saturation_amount);

        (r, g, b)
    }

    /// Apply curves adjustment
    fn apply_curves(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let mut r_out = r;
        let mut g_out = g;
        let mut b_out = b;

        // Apply master curve
        if let Some(master_curve) = self.config.curves.get("master") {
            if master_curve.enabled {
                r_out = self.evaluate_curve(master_curve, r_out);
                g_out = self.evaluate_curve(master_curve, g_out);
                b_out = self.evaluate_curve(master_curve, b_out);
            }
        }

        // Apply individual channel curves
        if let Some(red_curve) = self.config.curves.get("red") {
            if red_curve.enabled {
                r_out = self.evaluate_curve(red_curve, r_out);
            }
        }

        if let Some(green_curve) = self.config.curves.get("green") {
            if green_curve.enabled {
                g_out = self.evaluate_curve(green_curve, g_out);
            }
        }

        if let Some(blue_curve) = self.config.curves.get("blue") {
            if blue_curve.enabled {
                b_out = self.evaluate_curve(blue_curve, b_out);
            }
        }

        (r_out, g_out, b_out)
    }

    /// Evaluate curve at given input value
    fn evaluate_curve(&self, curve: &ColorCurve, input: f32) -> f32 {
        if curve.points.is_empty() {
            return input;
        }

        // Clamp input to curve range
        let input = input.clamp(0.0, 1.0);

        // Find surrounding points
        let mut left_idx = 0;
        let mut right_idx = curve.points.len() - 1;

        for (i, point) in curve.points.iter().enumerate() {
            if point.x <= input {
                left_idx = i;
            }
            if point.x >= input {
                right_idx = i;
                break;
            }
        }

        if left_idx == right_idx {
            return curve.points[left_idx].y;
        }

        let left_point = curve.points[left_idx];
        let right_point = curve.points[right_idx];

        if left_point.x == right_point.x {
            return left_point.y;
        }

        // Linear interpolation for now (can be extended to cubic, etc.)
        let t = (input - left_point.x) / (right_point.x - left_point.x);
        left_point.y + t * (right_point.y - left_point.y)
    }

    /// Apply levels adjustment
    fn apply_levels(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let levels = &self.config.levels;
        
        // Input levels
        let r = ((r - levels.input_black) / (levels.input_white - levels.input_black)).clamp(0.0, 1.0);
        let g = ((g - levels.input_black) / (levels.input_white - levels.input_black)).clamp(0.0, 1.0);
        let b = ((b - levels.input_black) / (levels.input_white - levels.input_black)).clamp(0.0, 1.0);

        // Gamma correction
        let r = r.powf(1.0 / levels.gamma);
        let g = g.powf(1.0 / levels.gamma);
        let b = b.powf(1.0 / levels.gamma);

        // Output levels
        let r = r * (levels.output_white - levels.output_black) + levels.output_black;
        let g = g * (levels.output_white - levels.output_black) + levels.output_black;
        let b = b * (levels.output_white - levels.output_black) + levels.output_black;

        (r, g, b)
    }

    /// Apply color wheels adjustment
    fn apply_color_wheels(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        // Simple color wheel implementation
        // This would be more sophisticated in a real implementation
        let wheels = &self.config.color_wheels;
        
        // Apply shadow color balance
        let shadow_factor = (1.0 - r.max(g).max(b)) * 0.5; // Darker areas affected more
        let r = r + wheels.shadows.x * shadow_factor;
        let g = g + wheels.shadows.y * shadow_factor;
        let b = b - (wheels.shadows.x + wheels.shadows.y) * shadow_factor * 0.5;

        // Apply highlight color balance
        let highlight_factor = r.max(g).max(b) * 0.5; // Brighter areas affected more
        let r = r + wheels.highlights.x * highlight_factor;
        let g = g + wheels.highlights.y * highlight_factor;
        let b = b - (wheels.highlights.x + wheels.highlights.y) * highlight_factor * 0.5;

        // Apply midtone color balance
        let midtone_factor = 1.0 - (r.max(g).max(b) - 0.5).abs() * 2.0; // Midtones affected most
        let r = r + wheels.midtones.x * midtone_factor;
        let g = g + wheels.midtones.y * midtone_factor;
        let b = b - (wheels.midtones.x + wheels.midtones.y) * midtone_factor * 0.5;

        (r, g, b)
    }

    /// Apply LUT transformation
    fn apply_lut(&self, r: f32, g: f32, b: f32, lut_name: &str) -> (f32, f32, f32) {
        // Find the active LUT
        if let Some(lut) = self.config.luts.iter().find(|l| l.name == lut_name) {
            // Simple 3D LUT sampling (would be more sophisticated in real implementation)
            let size = lut.size as f32;
            let r_idx = (r * (size - 1.0)) as usize;
            let g_idx = (g * (size - 1.0)) as usize;
            let b_idx = (b * (size - 1.0)) as usize;
            
            let index = (r_idx * lut.size as usize * lut.size as usize + g_idx * lut.size as usize + b_idx) * 3;
            
            if index + 2 < lut.data.len() {
                return (
                    lut.data[index],
                    lut.data[index + 1],
                    lut.data[index + 2],
                );
            }
        }
        
        (r, g, b)
    }

    /// Update histogram from frame data
    pub fn update_histogram(&mut self, pixels: &[u8], width: u32, height: u32) {
        if !self.real_time_analysis {
            return;
        }

        // Reset histogram
        self.histogram = HistogramData::default();

        // Analyze pixels
        for chunk in pixels.chunks_exact(4) {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            let a = chunk[3] as u32;

            // Skip transparent pixels
            if a == 0 {
                continue;
            }

            // Update RGB histograms
            self.histogram.rgb[0][r as usize] += 1;
            self.histogram.rgb[1][g as usize] += 1;
            self.histogram.rgb[2][b as usize] += 1;

            // Update combined RGB histogram
            let avg = ((r + g + b) / 3) as usize;
            self.histogram.rgb_combined[avg] += 1;

            // Update luma histogram (using Rec. 709 coefficients)
            let luma = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as usize;
            let luma = luma.min(255);
            self.histogram.luma[luma] += 1;

            // Update waveform (simplified)
            let x = (r % 256) as usize;
            let y = (255 - g) as usize;
            if x < 256 && y < 256 {
                self.histogram.waveform[y][x] += 1;
            }

            // Update vectorscope (simplified)
            let vec_x = ((r as f32 - b as f32) / 2.0 + 128.0) as usize;
            let vec_y = ((r as f32 + b as f32) / 2.0 - g as f32 + 128.0) as usize;
            if vec_x < 256 && vec_y < 256 {
                self.histogram.vectorscope[vec_y][vec_x] += 1;
            }
        }
    }

    /// Load LUT from file
    pub fn load_lut(&mut self, file_path: &str, name: &str) -> Result<(), Box<dyn Error>> {
        // This would implement actual LUT file parsing
        // For now, create a simple identity LUT
        let size = 16u32;
        let mut data = Vec::new();
        
        for r in 0..size {
            for g in 0..size {
                for b in 0..size {
                    let rf = r as f32 / (size - 1) as f32;
                    let gf = g as f32 / (size - 1) as f32;
                    let bf = b as f32 / (size - 1) as f32;
                    
                    data.push(rf);
                    data.push(gf);
                    data.push(bf);
                }
            }
        }

        let lut = LutInfo {
            name: name.to_string(),
            file_path: file_path.to_string(),
            size,
            format: LutFormat::Cube,
            data,
        };

        self.config.luts.push(lut);
        info!("Loaded LUT: {} ({}x{}x{})", name, size, size, size);

        Ok(())
    }

    /// Set active LUT
    pub fn set_active_lut(&mut self, lut_name: Option<String>) {
        self.config.active_lut = lut_name;
    }

    /// Reset all color grading settings
    pub fn reset_all(&mut self) {
        self.config = ColorGradingConfig::default();
        info!("Color grading settings reset to defaults");
    }

    /// Enable/disable color grading
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        info!("Color grading {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Get curve by name
    pub fn get_curve(&self, name: &str) -> Option<&ColorCurve> {
        self.config.curves.get(name)
    }

    /// Get mutable curve by name
    pub fn get_curve_mut(&mut self, name: &str) -> Option<&mut ColorCurve> {
        self.config.curves.get_mut(name)
    }

    /// Add curve point
    pub fn add_curve_point(&mut self, curve_name: &str, point: CurvePoint) -> Result<(), String> {
        if let Some(curve) = self.config.curves.get_mut(curve_name) {
            // Insert point while maintaining sorted order
            let insert_pos = curve.points.binary_search_by(|p| p.x.partial_cmp(&point.x).unwrap())
                .unwrap_or_else(|pos| pos);
            curve.points.insert(insert_pos, point);
            Ok(())
        } else {
            Err(format!("Curve '{}' not found", curve_name))
        }
    }

    /// Remove curve point
    pub fn remove_curve_point(&mut self, curve_name: &str, index: usize) -> Result<(), String> {
        if let Some(curve) = self.config.curves.get_mut(curve_name) {
            if index < curve.points.len() && !curve.points[index].locked {
                curve.points.remove(index);
                Ok(())
            } else {
                Err("Cannot remove locked point or invalid index".to_string())
            }
        } else {
            Err(format!("Curve '{}' not found", curve_name))
        }
    }
}

/// Color grading plugin
pub struct ColorGradingPlugin;

impl Plugin for ColorGradingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorGradingSystem>()
            .add_systems(Update, update_color_grading);
    }
}

/// Update color grading system
fn update_color_grading(
    mut color_grading: ResMut<ColorGradingSystem>,
    time: Res<Time>,
) {
    // Update any time-based color grading parameters here
    // This could include animated color corrections, auto-exposure, etc.
    
    if color_grading.enabled && color_grading.real_time_analysis {
        // Histogram analysis could be updated here if we had access to frame data
        // For now, this is a placeholder for future real-time analysis features
    }
}