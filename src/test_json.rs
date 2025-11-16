#[cfg(test)]
mod test_json {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "TYPE")]
    pub enum IsfInputType {
        #[serde(rename = "float")]
        Float { DEFAULT: f32, MIN: f32, MAX: f32 },
        #[serde(rename = "bool")]
        Bool { DEFAULT: bool },
        #[serde(rename = "color")]
        Color { DEFAULT: [f32; 4] },
        #[serde(rename = "point2D")]
        Point2D { DEFAULT: [f32; 2], MIN: [f32; 2], MAX: [f32; 2] },
        #[serde(rename = "image")]
        Image,
        #[serde(rename = "audio")]
        Audio,
        #[serde(rename = "audioFFT")]
        AudioFFT,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IsfInput {
        pub NAME: String,
        #[serde(flatten)]
        pub input_type: IsfInputType,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IsfMetadata {
        pub NAME: String,
        pub DESCRIPTION: Option<String>,
        pub CREDIT: Option<String>,
        pub ISFVSN: Option<String>,
        pub VSN: Option<String>,
        pub INPUTS: Option<Vec<IsfInput>>,
    }
    
    #[test]
    fn test_json_parsing() {
        let json_str = r#"{
                "NAME": "Basic Color",
                "DESCRIPTION": "Simple color shader",
                "INPUTS": [
                    {"NAME": "brightness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0}
                ]
            }"#;
        
        println!("Testing JSON:");
        println!("{}", json_str);
        
        match serde_json::from_str::<IsfMetadata>(json_str) {
            Ok(parsed) => println!("Parsed successfully: {:?}", parsed),
            Err(e) => println!("Parse error: {}", e),
        }
    }
}