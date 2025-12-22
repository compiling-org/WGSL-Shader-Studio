//! Documentation server for WGSL Shader Studio
//! This module provides a simple HTTP server to serve documentation files
//! from within the application.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Notify;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, StatusCode};
use hyper::server::Server;
use hyper_staticfile::Static;
use std::collections::HashMap;

/// Generate index.html content with links to all documentation
fn generate_index_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>WGSL Shader Studio Documentation</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        ul { list-style-type: none; padding: 0; }
        li { margin: 10px 0; }
        a { text-decoration: none; color: #0066cc; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <h1>WGSL Shader Studio Documentation</h1>
    <ul>
        <li><a href="./WGSL_FUNDAMENTALS.md">WGSL Fundamentals</a></li>
        <li><a href="./GLSL_FUNDAMENTALS.md">GLSL Fundamentals</a></li>
        <li><a href="./HLSL_FUNDAMENTALS.md">HLSL Fundamentals</a></li>
        <li><a href="./ISF_FUNDAMENTALS.md">ISF Fundamentals</a></li>
        <li><a href="./SHADER_CONVERSION_FRAMEWORK.md">Shader Conversion Framework</a></li>
        <li><a href="./APPLICATION_USAGE_GUIDE_COMPLETE.md">Application Usage Guide</a></li>
        <li><a href="./WGSL_SHADER_STUDIO_ARCHITECTURE.md">Technical Architecture</a></li>
        <li><a href="./ADVANCED_FEATURES.md">Advanced Features</a></li>
        <li><a href="./COMPREHENSIVE_DOCUMENTATION_INDEX.md">Comprehensive Documentation Index</a></li>
        <li><a href="./SHADER_STUDIO_COOKBOOK.md">Shader Studio Cookbook</a></li>
    </ul>
</body>
</html>"#.to_string()
}

/// Start the documentation server
pub async fn start_documentation_server(docs_path: &str) -> Result<(SocketAddr, Arc<Notify>), Box<dyn std::error::Error + Send + Sync>> {
    // Create a static file service for the docs directory
    let static_files = Static::new(Path::new(docs_path));
    
    // Create index.html content
    let index_content = generate_index_html();
    
    // Create a service that serves static files
    let make_svc = make_service_fn(move |_conn| {
        let static_files = static_files.clone();
        let index_content = index_content.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let static_files = static_files.clone();
                let index_content = index_content.clone();
                async move {
                    // Check if requesting root path
                    if req.uri().path() == "/" || req.uri().path() == "" {
                        // Return index.html content
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "text/html")
                            .body(Body::from(index_content))
                            .unwrap();
                        return Ok(response);
                    }
                    
                    // Serve the file
                    let response = static_files.serve(req).await;
                    
                    // Handle the result
                    match response {
                        Ok(mut resp) => {
                            // Add CORS headers for local development
                            resp.headers_mut().insert(
                                hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                                hyper::header::HeaderValue::from_static("*")
                            );
                            Ok(resp)
                        }
                        Err(_) => {
                            // Return a 404 response
                            let response = Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::from("Not Found"))
                                .unwrap();
                            Ok(response)
                        }
                    }
                }
            }))
        }
    });
    
    // Bind to localhost on a random port
    let addr: SocketAddr = ([127, 0, 0, 1], 0).into();
    let listener = TcpListener::bind(addr).await?;
    let addr = listener.local_addr()?;
    
    // Create a notify handle to signal shutdown
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    
    // Build the server
    let server = Server::from_tcp(listener.into_std()?)?
        .serve(make_svc)
        .with_graceful_shutdown(async move {
            notify_clone.notified().await;
        });
    
    // Spawn the server task
    tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("Documentation server error: {}", e);
        }
    });
    
    println!("Documentation server started at http://{}", addr);
    Ok((addr, notify))
}

/// Stop the documentation server
pub fn stop_documentation_server(notify: Arc<Notify>) {
    notify.notify_waiters();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_documentation_server_start() {
        // This is a basic test to ensure the server can start
        // In a real scenario, you would test actual HTTP requests
        let result = start_documentation_server("./docs").await;
        assert!(result.is_ok());
        
        let (addr, notify) = result.unwrap();
        assert_ne!(addr.port(), 0);
        
        // Stop the server
        stop_documentation_server(notify);
    }
}