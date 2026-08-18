// Fosfora integration module
// Provides audio feature structures and effect loading capabilities

pub mod features;
pub mod loader;
pub mod params;

pub use features::AudioFeatures;
pub use loader::EffectLoader;
pub use params::ParamDef;

// Import the effect format for proper .pfx parsing
use crate::control_engine::superinstance_bridge::{BudgetSummary, SuperInstanceController};
use crate::superinstance::conservation_integration::{BudgetTracker, ConservationEnforcer};
use crate::superinstance::flux_integration::FluxCompiler;
use crate::superinstance::plato_integration::{AlarmSeverity, PlatoRoom, SensorValue};
use std::path::Path;