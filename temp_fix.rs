            self.compile_wgsl_shader();
            self.start_preview_rendering();
            // Analyze shader uniforms using wgsl_bindgen
            self.analyze_shader_uniforms(&shader_name);
            
            println!("Selected shader: {}", shader_name);
        }
    }