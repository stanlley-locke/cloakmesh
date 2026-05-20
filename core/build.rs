fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "../proto/v1/cloakmesh.proto",
        "../proto/v1/cloak_service.proto",
        "../proto/v1/dht.proto",
        "../proto/v1/capability.proto",
        "../proto/v1/telemetry.proto",
        "../proto/v1/traffic.proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&protos, &["../proto/v1", "../proto"])?;
    println!("cargo:rerun-if-changed=../proto/");
    Ok(())
}
