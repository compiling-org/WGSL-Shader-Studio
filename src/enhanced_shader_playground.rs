use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderExample {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub wgsl_code: String,
    pub parameters: Vec<ParameterDefinition>,
    pub tags: Vec<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterDefinition {
    Float {
        name: String,
        default: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    Int {
        name: String,
        default: i32,
        min: i32,
        max: i32,
    },
    Bool {
        name: String,
        default: bool,
    },
    Vec2 {
        name: String,
        default: [f32; 2],
        min: [f32; 2],
        max: [f32; 2],
    },
    Vec3 {
        name: String,
        default: [f32; 3],
        min: [f32; 3],
        max: [f32; 3],
    },
    Vec4 {
        name: String,
        default: [f32; 4],
        min: [f32; 4],
        max: [f32; 4],
    },
    Color {
        name: String,
        default: [f32; 4],
    },
    Enum {
        name: String,
        default: String,
        options: Vec<String>,
    },
    Texture {
        name: String,
        default: Option<String>,
    },
}

#[derive(Resource, Default)]
pub struct ShaderPlaygroundState {
    pub examples: Vec<ShaderExample>,
    pub selected_example: Option<String>,
    pub search_query: String,
    pub selected_category: String,
    pub selected_difficulty: Option<Difficulty>,
    pub parameter_values: HashMap<String, ParameterValue>,
    pub show_examples_browser: bool,
    pub show_parameter_controls: bool,
    pub sort_by: SortBy,
    pub view_mode: ViewMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color([f32; 4]),
    Enum(String),
    Texture(Option<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Category,
    Difficulty,
    Date,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
    Compact,
}

impl Default for ShaderPlaygroundState {
    fn default() -> Self {
        Self {
            examples: create_default_examples(),
            selected_example: None,
            search_query: String::new(),
            selected_category: "All".to_string(),
            selected_difficulty: None,
            parameter_values: HashMap::new(),
            show_examples_browser: true,
            show_parameter_controls: true,
            sort_by: SortBy::Name,
            view_mode: ViewMode::Grid,
        }
    }
}

pub struct EnhancedShaderPlaygroundPlugin;

impl Plugin for EnhancedShaderPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShaderPlaygroundState>()
            .add_systems(Update, (
                examples_browser_system,
                parameter_controls_system,
                playground_toolbar_system,
            ));
    }
}

fn examples_browser_system(
    mut contexts: EguiContexts,
    mut state: ResMut<ShaderPlaygroundState>,
) {
    if !state.show_examples_browser {
        return;
    }

    egui::Window::new("Shader Examples Browser")
        .default_size([400.0, 600.0])
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            examples_browser_ui(ui, &mut state);
        });
}

fn examples_browser_ui(ui: &mut egui::Ui, state: &mut ShaderPlaygroundState) {
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.search_query)
            .hint_text("Search examples...");
        
        ui.menu_button("Sort", |ui| {
            ui.selectable_value(&mut state.sort_by, SortBy::Name, "Name");
            ui.selectable_value(&mut state.sort_by, SortBy::Category, "Category");
            ui.selectable_value(&mut state.sort_by, SortBy::Difficulty, "Difficulty");
            ui.selectable_value(&mut state.sort_by, SortBy::Date, "Date");
        });

        ui.menu_button("View", |ui| {
            ui.selectable_value(&mut state.view_mode, ViewMode::Grid, "Grid");
            ui.selectable_value(&mut state.view_mode, ViewMode::List, "List");
            ui.selectable_value(&mut state.view_mode, ViewMode::Compact, "Compact");
        });
    });

    ui.separator();

    ui.horizontal(|ui| {
        egui::ComboBox::from_label("Category")
            .selected_text(&state.selected_category)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.selected_category, "All".to_string(), "All");
                let categories: Vec<&str> = state.examples.iter()
                    .map(|e| e.category.as_str())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                
                for category in categories {
                    ui.selectable_value(&mut state.selected_category, category.to_string(), category);
                }
            });

        egui::ComboBox::from_label("Difficulty")
            .selected_text(format!("{:?}", state.selected_difficulty.unwrap_or(Difficulty::Beginner)))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.selected_difficulty, None, "All");
                ui.selectable_value(&mut state.selected_difficulty, Some(Difficulty::Beginner), "Beginner");
                ui.selectable_value(&mut state.selected_difficulty, Some(Difficulty::Intermediate), "Intermediate");
                ui.selectable_value(&mut state.selected_difficulty, Some(Difficulty::Advanced), "Advanced");
                ui.selectable_value(&mut state.selected_difficulty, Some(Difficulty::Expert), "Expert");
            });
    });

    ui.separator();

    let filtered_examples: Vec<&ShaderExample> = state.examples.iter()
        .filter(|example| {
            let matches_search = state.search_query.is_empty() || 
                example.name.to_lowercase().contains(&state.search_query.to_lowercase()) ||
                example.description.to_lowercase().contains(&state.search_query.to_lowercase()) ||
                example.tags.iter().any(|tag| tag.to_lowercase().contains(&state.search_query.to_lowercase()));
            
            let matches_category = state.selected_category == "All" || example.category == state.selected_category;
            
            let matches_difficulty = state.selected_difficulty.is_none() || 
                Some(example.difficulty) == state.selected_difficulty;
            
            matches_search && matches_category && matches_difficulty
        })
        .collect();

    match state.view_mode {
        ViewMode::Grid => examples_grid_ui(ui, state, &filtered_examples),
        ViewMode::List => examples_list_ui(ui, state, &filtered_examples),
        ViewMode::Compact => examples_compact_ui(ui, state, &filtered_examples),
    }
}

fn examples_grid_ui(ui: &mut egui::Ui, state: &mut ShaderPlaygroundState, examples: &[&ShaderExample]) {
    let columns = 3;
    egui::Grid::new("examples_grid")
        .num_columns(columns)
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            for (i, example) in examples.iter().enumerate() {
                if i > 0 && i % columns == 0 {
                    ui.end_row();
                }
                
                example_card_ui(ui, state, example);
            }
        });
}

fn examples_list_ui(ui: &mut egui::Ui, state: &mut ShaderPlaygroundState, examples: &[&ShaderExample]) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for example in examples {
            example_list_item_ui(ui, state, example);
        }
    });
}

fn examples_compact_ui(ui: &mut egui::Ui, state: &mut ShaderPlaygroundState, examples: &[&ShaderExample]) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for example in examples {
                if ui.button(&example.name).clicked() {
                    select_example(state, example);
                }
            }
        });
    });
}

fn example_card_ui(ui: &mut egui::Ui, state: &mut ShaderPlaygroundState, example: &ShaderExample) {
    ui.group(|ui| {
        ui.set_width(120.0);
        ui.set_height(150.0);
        
        ui.vertical(|ui| {
            if let Some(thumbnail) = &example.thumbnail {
                ui.image(egui::Image::new(egui::include_image!(thumbnail)))
                    .max_width(100.0)
                    .max_height(60.0);
            } else {
                ui.allocate_space([100.0, 60.0].into());
            }
            
            ui.label(&example.name);
            ui.label(&example.description);
            ui.label(format!("{:?}", example.difficulty));
            
            if ui.button("Load").clicked() {
                select_example(state, example);
            }
        });
    });
}

fn example_list_item_ui(ui: &mut egui::Ui, state: &mut ShaderPlaygroundState, example: &ShaderExample) {
    ui.horizontal(|ui| {
        ui.label(&example.name);
        ui.label(&example.category);
        ui.label(format!("{:?}", example.difficulty));
        ui.label(&example.description);
        
        if ui.button("Load").clicked() {
            select_example(state, example);
        }
    });
}

fn select_example(state: &mut ShaderPlaygroundState, example: &ShaderExample) {
    state.selected_example = Some(example.id.clone());
    state.parameter_values.clear();
    
    for param in &example.parameters {
        let value = match param {
            ParameterDefinition::Float { default, .. } => ParameterValue::Float(*default),
            ParameterDefinition::Int { default, .. } => ParameterValue::Int(*default),
            ParameterDefinition::Bool { default, .. } => ParameterValue::Bool(*default),
            ParameterDefinition::Vec2 { default, .. } => ParameterValue::Vec2(*default),
            ParameterDefinition::Vec3 { default, .. } => ParameterValue::Vec3(*default),
            ParameterDefinition::Vec4 { default, .. } => ParameterValue::Vec4(*default),
            ParameterDefinition::Color { default, .. } => ParameterValue::Color(*default),
            ParameterDefinition::Enum { default, .. } => ParameterValue::Enum(default.clone()),
            ParameterDefinition::Texture { default, .. } => ParameterValue::Texture(default.clone()),
        };
        
        state.parameter_values.insert(param.name().to_string(), value);
    }
}

fn parameter_controls_system(
    mut contexts: EguiContexts,
    mut state: ResMut<ShaderPlaygroundState>,
) {
    if !state.show_parameter_controls {
        return;
    }

    egui::Window::new("Parameter Controls")
        .default_size([300.0, 400.0])
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            if let Some(example_id) = &state.selected_example {
                if let Some(example) = state.examples.iter().find(|e| e.id == *example_id) {
                    parameter_controls_ui(ui, &mut state.parameter_values, &example.parameters);
                }
            } else {
                ui.label("No example selected");
            }
        });
}

fn parameter_controls_ui(
    ui: &mut egui::Ui,
    values: &mut HashMap<String, ParameterValue>,
    parameters: &[ParameterDefinition],
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for param in parameters {
            ui.horizontal(|ui| {
                match param {
                    ParameterDefinition::Float { name, min, max, step, .. } => {
                        ui.label(name);
                        if let Some(ParameterValue::Float(value)) = values.get_mut(name) {
                            ui.add(egui::DragValue::new(value)
                                .speed(*step)
                                .range(*min..=*max));
                        }
                    },
                    ParameterDefinition::Int { name, min, max, .. } => {
                        ui.label(name);
                        if let Some(ParameterValue::Int(value)) = values.get_mut(name) {
                            ui.add(egui::DragValue::new(value).range(*min..=*max));
                        }
                    },
                    ParameterDefinition::Bool { name, .. } => {
                        if let Some(ParameterValue::Bool(value)) = values.get_mut(name) {
                            ui.checkbox(value, name);
                        }
                    },
                    ParameterDefinition::Vec2 { name, min, max, .. } => {
                        ui.label(name);
                        if let Some(ParameterValue::Vec2(value)) = values.get_mut(name) {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut value[0])
                                    .speed(0.01).range(min[0]..=max[0]));
                                ui.add(egui::DragValue::new(&mut value[1])
                                    .speed(0.01).range(min[1]..=max[1]));
                            });
                        }
                    },
                    ParameterDefinition::Vec3 { name, min, max, .. } => {
                        ui.label(name);
                        if let Some(ParameterValue::Vec3(value)) = values.get_mut(name) {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut value[0])
                                    .speed(0.01).range(min[0]..=max[0]));
                                ui.add(egui::DragValue::new(&mut value[1])
                                    .speed(0.01).range(min[1]..=max[1]));
                                ui.add(egui::DragValue::new(&mut value[2])
                                    .speed(0.01).range(min[2]..=max[2]));
                            });
                        }
                    },
                    ParameterDefinition::Vec4 { name, min, max, .. } => {
                        ui.label(name);
                        if let Some(ParameterValue::Vec4(value)) = values.get_mut(name) {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut value[0])
                                    .speed(0.01).range(min[0]..=max[0]));
                                ui.add(egui::DragValue::new(&mut value[1])
                                    .speed(0.01).range(min[1]..=max[1]));
                                ui.add(egui::DragValue::new(&mut value[2])
                                    .speed(0.01).range(min[2]..=max[2]));
                                ui.add(egui::DragValue::new(&mut value[3])
                                    .speed(0.01).range(min[3]..=max[3]));
                            });
                        }
                    },
                    ParameterDefinition::Color { name, .. } => {
                        ui.label(name);
                        if let Some(ParameterValue::Color(value)) = values.get_mut(name) {
                            ui.color_edit_button_rgba(value);
                        }
                    },
                    ParameterDefinition::Enum { name, options, .. } => {
                        ui.label(name);
                        if let Some(ParameterValue::Enum(value)) = values.get_mut(name) {
                            egui::ComboBox::from_id_source(name)
                                .selected_text(value.clone())
                                .show_ui(ui, |ui| {
                                    for option in options {
                                        ui.selectable_value(value, option.clone(), option);
                                    }
                                });
                        }
                    },
                    ParameterDefinition::Texture { name, .. } => {
                        ui.label(name);
                        if let Some(ParameterValue::Texture(value)) = values.get_mut(name) {
                            if ui.button("Load Texture").clicked() {
                                // TODO: Implement texture loading dialog
                            }
                            if let Some(path) = value {
                                ui.label(path);
                            }
                        }
                    },
                }
            });
            ui.separator();
        }
    });
}

fn playground_toolbar_system(
    mut contexts: EguiContexts,
    mut state: ResMut<ShaderPlaygroundState>,
) {
    egui::TopBottomPanel::top("playground_toolbar")
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.toggle_value(&mut state.show_examples_browser, "📁 Examples");
                ui.toggle_value(&mut state.show_parameter_controls, "⚙️ Parameters");
                ui.separator();
                
                if ui.button("🔄 Reset Parameters").clicked() {
                    // Reset to default values
                    if let Some(example_id) = &state.selected_example {
                        if let Some(example) = state.examples.iter().find(|e| e.id == *example_id) {
                            state.parameter_values.clear();
                            for param in &example.parameters {
                                let value = match param {
                                    ParameterDefinition::Float { default, .. } => ParameterValue::Float(*default),
                                    ParameterDefinition::Int { default, .. } => ParameterValue::Int(*default),
                                    ParameterDefinition::Bool { default, .. } => ParameterValue::Bool(*default),
                                    ParameterDefinition::Vec2 { default, .. } => ParameterValue::Vec2(*default),
                                    ParameterDefinition::Vec3 { default, .. } => ParameterValue::Vec3(*default),
                                    ParameterDefinition::Vec4 { default, .. } => ParameterValue::Vec4(*default),
                                    ParameterDefinition::Color { default, .. } => ParameterValue::Color(*default),
                                    ParameterDefinition::Enum { default, .. } => ParameterValue::Enum(default.clone()),
                                    ParameterDefinition::Texture { default, .. } => ParameterValue::Texture(default.clone()),
                                };
                                state.parameter_values.insert(param.name().to_string(), value);
                            }
                        }
                    }
                }
                
                if ui.button("💾 Save Preset").clicked() {
                    // TODO: Implement preset saving
                }
                
                if ui.button("📤 Export").clicked() {
                    // TODO: Implement export functionality
                }
            });
        });
}

impl ParameterDefinition {
    fn name(&self) -> &str {
        match self {
            ParameterDefinition::Float { name, .. } => name,
            ParameterDefinition::Int { name, .. } => name,
            ParameterDefinition::Bool { name, .. } => name,
            ParameterDefinition::Vec2 { name, .. } => name,
            ParameterDefinition::Vec3 { name, .. } => name,
            ParameterDefinition::Vec4 { name, .. } => name,
            ParameterDefinition::Color { name, .. } => name,
            ParameterDefinition::Enum { name, .. } => name,
            ParameterDefinition::Texture { name, .. } => name,
        }
    }
}

fn create_default_examples() -> Vec<ShaderExample> {
    vec![
        ShaderExample {
            id: "basic_gradient".to_string(),
            name: "Basic Gradient".to_string(),
            category: "Basics".to_string(),
            description: "Simple color gradient shader".to_string(),
            difficulty: Difficulty::Beginner,
            wgsl_code: r#"
@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let color = vec3<f32>(uv.x, uv.y, 0.5);
    return vec4<f32>(color, 1.0);
}
"#.to_string(),
            parameters: vec![],
            tags: vec!["gradient".to_string(), "basic".to_string()],
            thumbnail: None,
        },
        ShaderExample {
            id: "plasma".to_string(),
            name: "Plasma Effect".to_string(),
            category: "Effects".to_string(),
            description: "Classic plasma effect with animated colors".to_string(),
            difficulty: Difficulty::Intermediate,
            wgsl_code: r#"
@group(0) @binding(0) var<uniform> time: f32;

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let v1 = sin(uv.x * 10.0 + time);
    let v2 = sin(uv.y * 10.0 + time * 1.5);
    let v3 = sin((uv.x + uv.y) * 10.0 + time * 0.5);
    let v4 = sin(distance(uv, vec2<f32>(0.5, 0.5)) * 10.0 + time * 2.0);
    
    let color = vec3<f32>(v1, v2, v3 + v4) * 0.5 + 0.5;
    return vec4<f32>(color, 1.0);
}
"#.to_string(),
            parameters: vec![
                ParameterDefinition::Float {
                    name: "speed".to_string(),
                    default: 1.0,
                    min: 0.1,
                    max: 5.0,
                    step: 0.1,
                },
                ParameterDefinition::Vec3 {
                    name: "color_scale".to_string(),
                    default: [1.0, 1.0, 1.0],
                    min: [0.0, 0.0, 0.0],
                    max: [2.0, 2.0, 2.0],
                },
            ],
            tags: vec!["plasma".to_string(), "animated".to_string(), "effect".to_string()],
            thumbnail: None,
        },
        ShaderExample {
            id: "noise".to_string(),
            name: "Procedural Noise".to_string(),
            category: "Procedural".to_string(),
            description: "Simple noise pattern generation".to_string(),
            difficulty: Difficulty::Advanced,
            wgsl_code: r#"
fn random(st: vec2<f32>) -> f32 {
    return fract(sin(dot(st.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453123);
}

fn noise(st: vec2<f32>) -> f32 {
    let i = floor(st);
    let f = fract(st);
    
    let a = random(i);
    let b = random(i + vec2<f32>(1.0, 0.0));
    let c = random(i + vec2<f32>(0.0, 1.0));
    let d = random(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let n = noise(uv * 10.0);
    return vec4<f32>(vec3<f32>(n), 1.0);
}
"#.to_string(),
            parameters: vec![
                ParameterDefinition::Float {
                    name: "scale".to_string(),
                    default: 10.0,
                    min: 1.0,
                    max: 50.0,
                    step: 1.0,
                },
                ParameterDefinition::Float {
                    name: "contrast".to_string(),
                    default: 1.0,
                    min: 0.1,
                    max: 3.0,
                    step: 0.1,
                },
            ],
            tags: vec!["noise".to_string(), "procedural".to_string(), "texture".to_string()],
            thumbnail: None,
        },
    ]
}

pub fn get_example_shader_code(state: &ShaderPlaygroundState) -> Option<String> {
    state.selected_example.as_ref().and_then(|id| {
        state.examples.iter().find(|e| e.id == *id).map(|e| e.wgsl_code.clone())
    })
}

pub fn get_example_parameters(state: &ShaderPlaygroundState) -> Vec<ParameterDefinition> {
    state.selected_example.as_ref().and_then(|id| {
        state.examples.iter().find(|e| e.id == *id).map(|e| e.parameters.clone())
    }).unwrap_or_default()
}

pub fn get_parameter_values(state: &ShaderPlaygroundState) -> HashMap<String, ParameterValue> {
    state.parameter_values.clone()
}

pub fn set_parameter_value(state: &mut ShaderPlaygroundState, name: String, value: ParameterValue) {
    state.parameter_values.insert(name, value);
}