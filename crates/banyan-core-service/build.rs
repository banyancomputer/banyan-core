use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

fn report_build_profile() {
    println!(
        "cargo:rustc-env=BUILD_PROFILE={}",
        std::env::var("PROFILE").unwrap()
    );
}

fn report_enabled_features() {
    let mut enabled_features: Vec<&str> = Vec::new();

    // NOTE: When features are added or removed, they need to be manually listed here
    //#[cfg(feature = "development")]
    //enabled_features.push("development");

    // Mostly here to prevent a mut warning from showing up with no available features but also
    // kind of useful...
    if enabled_features.is_empty() {
        enabled_features.push("none");
    }

    println!(
        "cargo:rustc-env=BUILD_FEATURES={}",
        enabled_features.join(",")
    );
}

fn report_repository_version() {
    let git_describe = std::process::Command::new("git")
        .args(["describe", "--always", "--dirty", "--long", "--tags"])
        .output()
        .unwrap();

    let long_version = String::from_utf8(git_describe.stdout).unwrap();
    println!("cargo:rustc-env=REPO_VERSION={}", long_version);

    let build_timestamp = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={build_timestamp}");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=dist/pricing.ron");
    println!("cargo:rerun-if-changed=migrations");

    report_repository_version();
    report_build_profile();
    report_enabled_features();
}
