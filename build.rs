use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::configure()// Specify where the generated files will go
        .compile_protos(
            &["proto/user.proto"], // Path to your proto file
            &["proto"],          // Path to the directory containing proto files
        )?;
    println!("cargo:rerun-if-changed=proto/user.proto");
    Ok(())
}