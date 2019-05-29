use failure::Error;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{Read};
use std::path::{Path, PathBuf};

const WHITELISTED_FUNCTIONS: &[&str] = &[
    "SSL_CTX_new",
    "SSL_CTX_set_cipher_list",
    "SSL_CTX_set_ciphersuites",
    "SSL_CTX_set_default_verify_paths",
    "SSL_CTX_set_tlsext_servername_callback",
    "SSL_CTX_use_PrivateKey",
    "TLS_method",
];

#[derive(Deserialize)]
struct Build {
    crypto_sources: Vec<PathBuf>,

    #[serde(default)]
    #[cfg_attr(
        all(target_os = "ios", target_arch = "aarch64"),
        serde(rename = "crypto_sources_ios_aarch64")
    )]
    #[cfg_attr(
        all(target_os = "ios", target_arch = "arm"),
        serde(rename = "crypto_sources_ios_arm")
    )]
    #[cfg_attr(
        all(target_os = "linux", target_arch = "aarch64"),
        serde(rename = "crypto_sources_linux_aarch64")
    )]
    #[cfg_attr(
        all(target_os = "linux", target_arch = "arm"),
        serde(rename = "crypto_sources_linux_arm")
    )]
    #[cfg_attr(
        all(
            target_os = "linux",
            target_arch = "powerpc64",
            target_endian = "little"
        ),
        serde(rename = "crypto_sources_linux_ppc64le")
    )]
    #[cfg_attr(
        all(target_os = "linux", target_arch = "x86"),
        serde(rename = "crypto_sources_linux_x86")
    )]
    #[cfg_attr(
        all(target_os = "linux", target_arch = "x86_64"),
        serde(rename = "crypto_sources_linux_x86_64")
    )]
    #[cfg_attr(
        all(target_os = "macos", target_arch = "x86"),
        serde(rename = "crypto_sources_mac_x86")
    )]
    #[cfg_attr(
        all(target_os = "macos", target_arch = "x86_64"),
        serde(rename = "crypto_sources_mac_x86_64")
    )]
    #[cfg_attr(
        all(target_os = "windows", target_arch = "x86"),
        serde(rename = "crypto_sources_win_x86")
    )]
    #[cfg_attr(
        all(target_os = "windows", target_arch = "x86_64"),
        serde(rename = "crypto_sources_win_x86_64")
    )]
    crypto_sources_asm: Vec<PathBuf>,

    fips_fragments: Vec<PathBuf>,

    ssl_sources: Vec<PathBuf>,
}

fn main() -> Result<(), Error> {
    let mut build_file = File::open("third_party/boringssl/BUILD.generated.bzl")?;
    let mut build_source = Vec::new();
    build_file.read_to_end(&mut build_source)?;

    // The Bazel file looks like valid TOML.
    let build = toml::from_slice::<Build>(build_source.as_slice())?;

    let mut libcrypto = cc::Build::new();
    libcrypto.flag("-std=c11");

    let mut libssl = cc::Build::new();
    libssl.cpp(true).flag("-std=c++11");

    for build in &mut [&mut libcrypto, &mut libssl] {
        build
            .include("third_party/boringssl/src/include")
            .define("BORINGSSL_IMPLEMENTATION", None)
            .warnings(false);
    }

    let src_root = Path::new("third_party/boringssl");

    for source in build.crypto_sources {
        libcrypto.file(src_root.join(source));
    }
    for source in build.fips_fragments {
        libcrypto.file(src_root.join(source));
    }
    for source in build.crypto_sources_asm {
        libcrypto.file(src_root.join(source));
    }

    for source in build.ssl_sources {
        libssl.file(src_root.join(source));
    }

    libcrypto.compile("libcrypto.a");
    libssl.compile("libssl.a");

    let builder = bindgen::Builder::default()
        .clang_arg("-Ithird_party/boringssl/src/include")
        .blacklist_type(r#"_+darwin_.*"#)
        .blacklist_type(r#"_+opaque_.*"#)
        .blacklist_type("pthread_rwlock_t");

    let builder = WHITELISTED_FUNCTIONS
        .iter()
        .fold(builder, |builder, function| {
            builder.whitelist_function(function)
        });

    let bindings = builder
        .header("wrapper.h")
        .generate()
        .expect("failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
