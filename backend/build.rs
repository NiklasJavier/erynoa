use axum_connect_build::{axum_connect_codegen, AxumConnectGenSettings};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate Rust code from proto files using axum-connect-build
    // Note: proto directory is now in backend/proto
    // In Docker: /workspace/backend/proto
    // On host: proto (relative to backend/)
    let proto_dir = if std::path::Path::new("/workspace/backend/proto").exists() {
        "/workspace/backend/proto"
    } else {
        "proto"
    };

    let settings = AxumConnectGenSettings::from_directory_recursive(proto_dir)
        .expect("failed to glob proto files");
    axum_connect_codegen(settings)?;
    Ok(())
}
