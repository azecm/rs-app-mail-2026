// use std::env;
use std::process::Command;

// touch build.rs && cargo build -vv
fn main() {
    // Tell Cargo to re-run this script only if frontend files change
    //println!("cargo::rerun-if-changed=../frontend-ui/src");
    //println!("cargo::rerun-if-changed=../frontend-ui/index.html");
    //
    // trunk build --release --dist release
    //println!("++");
    println!("cargo::warning=STARTED");
    /*Command::new("echo")
    .arg("123")
    .status()
    .expect("failed to spawn process");*/

    // Execute Trunk build inside the frontend folder
    let status = Command::new("trunk")
        .args(["build", "--release", "--dist", "release"])
        .current_dir("../client-leptos")
        // CRITICAL: Prevent Cargo environmental variable leakage
        .env_remove("RUSTC_LINKER")
        .env_remove("TARGET")
        .status()
        .expect("Failed to execute Trunk build process");

    if !status.success() {
        panic!("Trunk build failed with status: {}", status);
    }

    gen_csp();
}

fn gen_csp() {
    println!("cargo::warning=CSP");
    use base64::Engine;
    use sha2::{Digest, Sha256};
    let Ok(source) = std::fs::read_to_string("../client-leptos/release/index.html") else {
        panic!("scp - read index.html error");
    };
    let script_begin = "<script type=module>";
    let script_end = "</script>";
    if let Some(start) = source.find(script_begin)
        && let Some(end) = source.find(script_end)
    {
        let script = &source[start + script_begin.len()..end];

        let mut hasher = Sha256::new();
        hasher.update(script);
        let hash = hasher.finalize();

        // 2. Encode the hash as a Base64 string
        let base64_hash = base64::engine::general_purpose::STANDARD.encode(hash);

        // 3. Format into the correct CSP directive
        let csp_source = format!("sha256-{}", base64_hash);

        if let Err(err) = std::fs::write(
            "../client-leptos/static/csp.conf",
            format!(
                "set $CSP \"default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'wasm-unsafe-eval' '{csp_source}';\";"
            ),
        ) {
            panic!("scp - write csp.conf {err:?}");
        }

        println!("CSP Source(1): {}", csp_source);
        println!("cargo::warning=CSP::{}", csp_source);
    }
}
