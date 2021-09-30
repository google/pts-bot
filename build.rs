use tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(
            &[
                "proto/common.proto",
                "proto/rootservice.proto",
                "proto/l2cap.proto",
                "proto/neighbor.proto",
            ],
            &["."],
        )?;
    Ok(())
}
