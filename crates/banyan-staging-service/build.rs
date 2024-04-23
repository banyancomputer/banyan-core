fn report_build_profile() {
    println!(
        "cargo:rustc-env=BUILD_PROFILE={}",
        std::env::var("PROFILE").unwrap()
    );
}

fn report_enabled_features() {
    let mut enabled_features: Vec<&str> = Vec::new();

    if cfg!(feature = "sqlite") {
        enabled_features.push("sqlite");
    }

    if enabled_features.is_empty() {
        enabled_features.push("none");
    }

    println!(
        "cargo:rustc-env=BUILD_FEATURES={}",
        enabled_features.join(",")
    );
}

fn report_repository_version() {
    // attempt to get from env vars, fall back to command execution; relevant in docker build
    let git_describe = if let Ok(ci_build_ref) = std::env::var("CI_BUILD_REF") {
        ci_build_ref
    } else {
        String::from_utf8(
            std::process::Command::new("git")
                .args(["describe", "--always", "--dirty", "--long", "--tags"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
    };

    println!("cargo:rustc-env=REPO_VERSION={}", git_describe);
}

fn main() {
    // When this script changes, the changes will likely affect the produced binary (or why is it
    // being done here)? Not sure why this isn't the default but here it is...
    println!("cargo:rerun-if-changed=build.rs");

    // Migrations lie outside of our normal source but are both embedded in our code, and our code
    // is dependent on the changes they represent. We should be building when this changes
    println!("cargo:rerun-if-changed=migrations");

    report_build_profile();
    report_enabled_features();
    report_repository_version();
}
