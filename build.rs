use std::process::Command;

fn main() {
    Command::new("echo").arg("Building UI").status().unwrap();
    println!("cargo:rerun-if-changed=./ui/package.json");
    println!("cargo:rerun-if-changed=ui/package-lock.json");
    println!("cargo:rerun-if-changed=ui/build.rs");

    let npm_ci_status = Command::new("npm")
        .arg("ci")
        .current_dir("ui")
        .status()
        .expect("failed to execute npm ci");
    if !npm_ci_status.success() {
        panic!("npm ci failed with exit code: {}", npm_ci_status);
    }
    let build_client_status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("ui") // Specify the directory
        .status()
        .expect("failed to execute npm run build");

    if !build_client_status.success() {
        panic!(
            "npm run build failed with exit code: {}",
            build_client_status
        );
    }
}
