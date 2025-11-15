    // Missing method implementations that were referenced but not defined
    pub fn save_theme_settings(&mut self) {
        let theme_data = serde_json::json!({
            "current_theme": self.current_theme,
            "custom_colors": self.custom_theme_colors,
            "brightness": self.brightness,
            "contrast": self.contrast,
            "font_size": self.font_size,
        });
        
        if let Ok(json) = serde_json::to_string_pretty(&theme_data) {
            let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let theme_file = config_dir.join("wgsl-shader-studio").join("theme.json");
            
            if let Some(parent) = theme_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            
            match std::fs::write(&theme_file, json) {
                Ok(_) => println!("✓ Theme settings saved"),
                Err(e) => eprintln!("✗ Failed to save theme settings: {}", e),
            }
        }
    }

    pub fn load_theme_settings(&mut self) {
        let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let theme_file = config_dir.join("wgsl-shader-studio").join("theme.json");
        
        if theme_file.exists() {
            match std::fs::read_to_string(&theme_file) {
                Ok(json) => {
                    if let Ok(theme_data) = serde_json::from_str::<serde_json::Value>(&json) {
                        if let Some(theme) = theme_data["current_theme"].as_str() {
                            self.current_theme = theme.to_string();
                        }
                        if let Some(brightness) = theme_data["brightness"].as_f64() {
                            self.brightness = brightness as f32;
                        }
                        if let Some(contrast) = theme_data["contrast"].as_f64() {
                            self.contrast = contrast as f32;
                        }
                        if let Some(font_size) = theme_data["font_size"].as_f64() {
                            self.font_size = font_size as f32;
                        }
                        println!("✓ Theme settings loaded");
                    }
                }
                Err(e) => eprintln!("✗ Failed to load theme settings: {}", e),
            }
        }
    }