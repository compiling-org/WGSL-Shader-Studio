// Fosfora integration module
// Provides audio feature structures and effect loading capabilities

pub mod features;
pub mod loader;
pub mod params;

pub use features::AudioFeatures;
pub use loader::EffectLoader;
pub use params::ParamDef;