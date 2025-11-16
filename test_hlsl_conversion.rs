use wgsl_shader_studio::converter::hlsl::HLSLConverter;

fn main() {
    println!("Testing HLSL to WGSL conversion...");
    
    let mut converter = HLSLConverter::new().unwrap();
    
    let hlsl_code = r#"
        cbuffer Constants {
            float4x4 worldViewProj;
            float4 lightDir;
        };
        
        Texture2D diffuseTexture;
        SamplerState diffuseSampler;
        
        float4 main(float4 pos : POSITION) : SV_POSITION {
            return mul(pos, worldViewProj);
        }
    "#;
    
    match converter.convert(hlsl_code, "test.hlsl") {
        Ok(wgsl_code) => {
            println!("✓ HLSL conversion successful!");
            println!("Generated WGSL:");
            println!("{}", wgsl_code);
        }
        Err(e) => {
            println!("✗ HLSL conversion failed: {}", e);
        }
    }
}