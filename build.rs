use std::{fs, io, path::PathBuf};

use prost_build::Config;
use prost_types::FileDescriptorSet;

fn collect_protos(dir: &str) -> io::Result<Vec<PathBuf>> {
    fs::read_dir(dir)?
        .map(|dir_entry| {
            let path = dir_entry?.path();
            assert!(path.is_file());
            assert!(path.extension().is_some_and(|ext| ext == "proto"));
            Ok(path)
        })
        .collect::<io::Result<Vec<_>>>()
}

#[cfg(any(feature = "deadlock", feature = "dota2"))]
type ExternDefs<'a> = (&'a FileDescriptorSet, &'static str);

/// declares all enums and messages from ExternDefs' [`FileDescriptorSet`] as external. more info
/// is available in documentation of [`prost_build::config::Config::extern_path`].
#[cfg(any(feature = "deadlock", feature = "dota2"))]
fn decl_externs(externs: &[ExternDefs], config: &mut Config) {
    use std::collections::HashSet;

    // NOTE: prost runs heck's upper camel case transformer on all idents. valve-defined names such
    // as EGCPlatform will be transformed into EgcPlatform, etc.
    // see https://github.com/tokio-rs/prost/blob/9ed944eb633480079037dfceeee61aac6cd0c94f/prost-build/src/ident.rs#L30
    use heck::ToUpperCamelCase;

    let mut declared: HashSet<String> = Default::default();
    for (fds, rust_path) in externs {
        for file in &fds.file {
            file.enum_type
                .iter()
                .map(|enum_type| enum_type.name())
                .chain(
                    file.message_type
                        .iter()
                        .map(|message_type| message_type.name()),
                )
                .for_each(|name| {
                    if declared.contains(name) {
                        return;
                    }
                    config.extern_path(
                        format!(".{}", name),
                        format!("{}::{}", rust_path, name.to_upper_camel_case()),
                    );
                    declared.insert(name.to_string());
                });
        }
    }
}

fn load_common_protos() -> io::Result<(FileDescriptorSet, Config)> {
    let mut config = Config::default();
    config.default_package_filename("common");

    let protos = collect_protos("protos/common")?;
    Ok((config.load_fds(&protos, &["protos/common"])?, config))
}

fn load_gcsdk_protos() -> io::Result<(FileDescriptorSet, Config)> {
    let mut config = Config::default();
    config.default_package_filename("gcsdk");

    let protos = collect_protos("protos/gcsdk")?;
    Ok((config.load_fds(&protos, &["protos/gcsdk"])?, config))
}

#[cfg(feature = "deadlock")]
fn compile_deadlock_protos(externs: &[ExternDefs]) -> io::Result<()> {
    let mut config = Config::default();
    config.default_package_filename("deadlock");

    decl_externs(externs, &mut config);

    let protos = collect_protos("protos/deadlock")?;
    config.compile_protos(
        &protos,
        &["protos/deadlock", "protos/gcsdk", "protos/common"],
    )
}

#[cfg(feature = "dota2")]
fn compile_dota2_protos(externs: &[ExternDefs]) -> io::Result<()> {
    let mut config = Config::default();
    config.default_package_filename("dota2");

    decl_externs(externs, &mut config);

    let protos = collect_protos("protos/dota2")?;
    config.compile_protos(&protos, &["protos/dota2", "protos/gcsdk", "protos/common"])
}

fn main() -> io::Result<()> {
    // tell cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=protos");

    #[cfg(feature = "protobuf-src")]
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let (common_fds, mut common_config) = load_common_protos()?;
    common_config.compile_fds(common_fds.clone())?;

    let (gcsdk_fds, mut gcsdk_config) = load_gcsdk_protos()?;
    gcsdk_config.compile_fds(gcsdk_fds.clone())?;

    #[cfg(feature = "deadlock")]
    compile_deadlock_protos(&[(&common_fds, "crate::common"), (&gcsdk_fds, "crate::gcsdk")])?;

    #[cfg(feature = "dota2")]
    compile_dota2_protos(&[(&common_fds, "crate::common")])?;

    Ok(())
}
