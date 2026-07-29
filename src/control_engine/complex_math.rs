//! # Complex-Valued Control Operations
//!
//! Provides complex-valued math for transforming signals (audio, MIDI, OSC, sensors)
//! into shader control vectors. Uses `num-complex` for `Complex32` (f32-based complex numbers).
//!
//! ## Design
//!
//! Complex numbers encode **phase + magnitude** naturally, which maps well to:
//! - **Oscillatory fields**: rotations, waves, LFOs with phase coherence
//! - **Spectral palettes**: audio frequency bands → color/geometry morph
//! - **Flow fields**: vector fields with curl/divergence from complex potentials
//!
//! All computation happens in Rust. Outputs are real-valued `Vec<f32>` control vectors
//! that are piped to WGSL uniforms/buffers.

use num_complex::Complex32;

/// Frequency band characteristics for spectral processing
#[derive(Debug, Clone)]
pub struct FrequencyBand {
    /// Center frequency in Hz
    pub center: f32,
    /// Bandwidth in Hz
    pub bandwidth: f32,
    /// Current magnitude (0.0 - 1.0)
    pub magnitude: f32,
    /// Current phase in radians
    pub phase: f32,
}

/// A bank of oscillators that can be driven by complex modulation
#[derive(Debug, Clone)]
pub struct OscillatorBank {
    /// Individual oscillators, each with frequency, phase, amplitude
    oscillators: Vec<Oscillator>,
}

/// A single oscillator with complex state
#[derive(Debug, Clone)]
pub struct Oscillator {
    /// Frequency in Hz
    pub frequency: f32,
    /// Current phase in radians
    pub phase: f32,
    /// Amplitude (0.0 - 1.0)
    pub amplitude: f32,
    /// Modulation depth for complex coupling
    pub modulation_depth: f32,
}

impl Oscillator {
    /// Create a new oscillator
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            phase: 0.0,
            amplitude: 0.5,
            modulation_depth: 0.0,
        }
    }

    /// Tick the oscillator forward by dt seconds
    /// Returns the current complex value as a (real, imag) pair
    pub fn tick(&mut self, dt: f32) -> (f32, f32) {
        self.phase += self.frequency * dt * std::f32::consts::TAU;
        // Normalize phase to [0, TAU)
        if self.phase > std::f32::consts::TAU {
            self.phase -= std::f32::consts::TAU;
        }
        let c = Complex32::new(0.0, self.phase).exp();
        (c.re * self.amplitude, c.im * self.amplitude)
    }

    /// Get the current magnitude
    pub fn magnitude(&self) -> f32 {
        self.amplitude
    }

    /// Get the current phase angle
    pub fn phase_angle(&self) -> f32 {
        self.phase
    }
}

impl OscillatorBank {
    /// Create an oscillator bank with N oscillators at harmonic ratios
    ///
    /// Oscillators are tuned to: base_freq, 2*base_freq, 3*base_freq, ...
    /// Ratios follow a prime-like pattern for non-repeating modulation.
    pub fn new(count: usize, base_freq: f32) -> Self {
        let ratios: Vec<f32> = [1.0, 2.0, 3.0, 5.0, 7.0, 11.0, 13.0, 17.0]
            .iter()
            .take(count)
            .copied()
            .collect();
        let oscillators: Vec<Oscillator> = ratios
            .iter()
            .map(|&r| Oscillator::new(base_freq * r))
            .collect();
        Self { oscillators }
    }

    /// Create a bank with explicit frequency ratios
    pub fn with_ratios(base_freq: f32, ratios: &[f32]) -> Self {
        let oscillators: Vec<Oscillator> = ratios
            .iter()
            .map(|&r| Oscillator::new(base_freq * r))
            .collect();
        Self { oscillators }
    }

    /// Tick all oscillators and return a combined control vector
    ///
    /// Returns Vec<f32> with interleaved real/imag values:
    /// [osc0_re, osc0_im, osc1_re, osc1_im, ...]
    pub fn tick_all(&mut self, dt: f32) -> Vec<f32> {
        let mut output = Vec::with_capacity(self.oscillators.len() * 2);
        for osc in &mut self.oscillators {
            let (re, im) = osc.tick(dt);
            output.push(re);
            output.push(im);
        }
        output
    }

    /// Get the count of oscillators
    pub fn len(&self) -> usize {
        self.oscillators.len()
    }

    /// Check if the bank is empty
    pub fn is_empty(&self) -> bool {
        self.oscillators.is_empty()
    }

    /// Set amplitude for all oscillators
    pub fn set_global_amplitude(&mut self, amp: f32) {
        for osc in &mut self.oscillators {
            osc.amplitude = amp;
        }
    }
}

/// Convert a complex number to a phase-magnitude control pair
pub fn complex_to_control(c: Complex32) -> (f32, f32) {
    (c.norm(), c.arg())
}

/// Create a rotation matrix from a complex number
/// Useful for transforming 2D UV coordinates or vectors
pub fn complex_rotation(phase: f32) -> [f32; 4] {
    let cos = phase.cos();
    let sin = phase.sin();
    [cos, -sin, sin, cos]
}

/// Create a phasor (complex exponential) from a phase angle
pub fn phasor(phase: f32) -> Complex32 {
    Complex32::new(0.0, phase).exp()
}

/// Map a complex value to a color vector (RGB)
///
/// phase → hue, magnitude → saturation/value
pub fn complex_to_color(c: Complex32) -> [f32; 3] {
    let magnitude = c.norm().min(1.0);
    let phase = c.arg();
    // Normalize phase from [-PI, PI] to [0, 1] for hue
    let hue = (phase / std::f32::consts::TAU + 0.5).fract();
    // Simple hue-to-RGB conversion
    let r = (hue + 0.333).fract();
    let g = (hue + 0.667).fract();
    let b = hue;
    [
        (1.0 - (r * 6.0 - 3.0).abs()).clamp(0.0, 1.0) * magnitude,
        (1.0 - (g * 6.0 - 3.0).abs()).clamp(0.0, 1.0) * magnitude,
        (1.0 - (b * 6.0 - 3.0).abs()).clamp(0.0, 1.0) * magnitude,
    ]
}

/// Apply complex rotation to a 2D vector
pub fn rotate_vec2(v: [f32; 2], phase: f32) -> [f32; 2] {
    let c = phase.cos();
    let s = phase.sin();
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c]
}

/// Create a complex field from frequency band magnitudes and phases
///
/// Returns a vector of control values suitable for shader uniforms.
/// Each band contributes: [magnitude * cos(phase), magnitude * sin(phase)]
pub fn frequency_bands_to_control(bands: &[FrequencyBand]) -> Vec<f32> {
    let mut control = Vec::with_capacity(bands.len() * 2);
    for band in bands {
        control.push(band.magnitude * band.phase.cos());
        control.push(band.magnitude * band.phase.sin());
    }
    control
}

/// Simple complex low-pass filter for smoothing control signals
#[derive(Debug, Clone)]
pub struct ComplexSmoother {
    /// Smoothing factor (0.0 = no smoothing, 1.0 = fully smoothed)
    pub alpha: f32,
    /// Previous complex output
    prev: Complex32,
}

impl ComplexSmoother {
    /// Create a new smoother
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            prev: Complex32::new(0.0, 0.0),
        }
    }

    /// Apply smoothing to a complex input
    pub fn smooth(&mut self, input: Complex32) -> Complex32 {
        let output = self.prev * self.alpha + input * (1.0 - self.alpha);
        self.prev = output;
        output
    }

    /// Apply smoothing to real-valued signals (treat as complex with zero imag)
    pub fn smooth_real(&mut self, input: f32) -> f32 {
        let c = Complex32::new(input, 0.0);
        let smoothed = self.smooth(c);
        smoothed.re
    }

    /// Reset smoother state
    pub fn reset(&mut self) {
        self.prev = Complex32::new(0.0, 0.0);
    }
}

/// Map a control vector to shader uniform values
///
/// Takes the raw outputs from complex math and transforms them
/// to the expected ranges for WGSL uniforms (typically 0-1 or -1 to 1).
pub fn map_to_uniform_range(
    control: &[f32],
    mode: UniformMapping,
) -> Vec<f32> {
    match mode {
        UniformMapping::ZeroToOne => {
            control.iter().map(|&v| (v * 0.5 + 0.5).clamp(0.0, 1.0)).collect()
        }
        UniformMapping::MinusOneToOne => {
            control.iter().map(|&v| v.clamp(-1.0, 1.0)).collect()
        }
        UniformMapping::Raw => control.to_vec(),
    }
}

/// How to map control values to shader uniform ranges
#[derive(Debug, Clone, Copy)]
pub enum UniformMapping {
    /// Map from [-1, 1] range to [0, 1] range (for color, opacity, etc.)
    ZeroToOne,
    /// Keep in [-1, 1] range (for offsets, directions, etc.)
    MinusOneToOne,
    /// Pass through raw values unchanged
    Raw,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_tick() {
        let mut osc = Oscillator::new(1.0); // 1 Hz
        let (re, im) = osc.tick(0.25); // 1/4 cycle
        assert!((re.abs() - 0.0).abs() < 0.001);
        assert!((im - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_oscillator_bank() {
        let mut bank = OscillatorBank::new(3, 1.0);
        assert_eq!(bank.len(), 3);
        let output = bank.tick_all(1.0 / 60.0);
        assert_eq!(output.len(), 6); // 3 oscs * 2 (re, im)
    }

    #[test]
    fn test_complex_to_control() {
        let c = Complex32::new(1.0, 0.0);
        let (mag, phase) = complex_to_control(c);
        assert!((mag - 1.0).abs() < 0.001);
        assert!((phase - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_complex_rotation() {
        let rot = complex_rotation(0.0);
        assert!((rot[0] - 1.0).abs() < 0.001); // cos(0) = 1
        assert!((rot[1] - 0.0).abs() < 0.001); // -sin(0) = 0
    }

    #[test]
    fn test_phasor() {
        let p = phasor(0.0);
        assert!((p.re - 1.0).abs() < 0.001);
        assert!((p.im - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_complex_to_color() {
        let c = Complex32::new(1.0, 0.0);
        let color = complex_to_color(c);
        assert!(color[0] >= 0.0 && color[0] <= 1.0);
        assert!(color[1] >= 0.0 && color[1] <= 1.0);
        assert!(color[2] >= 0.0 && color[2] <= 1.0);
    }

    #[test]
    fn test_rotate_vec2() {
        let v = [1.0, 0.0];
        let rotated = rotate_vec2(v, std::f32::consts::FRAC_PI_2);
        assert!((rotated[0] - 0.0).abs() < 0.001);
        assert!((rotated[1] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_complex_smoother() {
        let mut smoother = ComplexSmoother::new(0.9);
        let input = Complex32::new(1.0, 0.0);
        let output1 = smoother.smooth(input);
        assert!((output1.re - 0.9).abs() < 0.01);
        let output2 = smoother.smooth(input);
        assert!((output2.re - 0.99).abs() < 0.01);
    }

    #[test]
    fn test_smooth_real() {
        let mut smoother = ComplexSmoother::new(0.8);
        let s1 = smoother.smooth_real(1.0);
        assert!((s1 - 0.8).abs() < 0.01);
        let s2 = smoother.smooth_real(0.0);
        assert!((s2 - 0.64).abs() < 0.01);
    }

    #[test]
    fn test_frequency_bands_to_control() {
        let bands = vec![
            FrequencyBand {
                center: 100.0,
                bandwidth: 50.0,
                magnitude: 0.5,
                phase: 0.0,
            },
            FrequencyBand {
                center: 200.0,
                bandwidth: 50.0,
                magnitude: 1.0,
                phase: std::f32::consts::FRAC_PI_2,
            },
        ];
        let control = frequency_bands_to_control(&bands);
        assert_eq!(control.len(), 4);
        assert!((control[0] - 0.5).abs() < 0.001);
        assert!((control[2] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_uniform_mapping() {
        let raw = vec![-0.5, 0.0, 0.5];
        let mapped = map_to_uniform_range(&raw, UniformMapping::ZeroToOne);
        assert!((mapped[0] - 0.25).abs() < 0.001);
        assert!((mapped[1] - 0.5).abs() < 0.001);
        assert!((mapped[2] - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_harmonic_bank_ratios() {
        let bank = OscillatorBank::new(4, 100.0);
        // Should have 4 oscillators at 100, 200, 300, 500 Hz
        assert_eq!(bank.len(), 4);
    }
}
