fn main() {
    // let src = "../../docs/openapi/core-service-client-spec.json";
    // let file = std::fs::File::open(src).unwrap();
    // let spec = serde_json::from_reader(file).unwrap();
    // let mut generator = progenitor::Generator::default();
    //
    // let tokens = generator.generate_tokens(&spec).unwrap();
    // let ast = syn::parse2(tokens).unwrap();
    // let content = prettyplease::unparse(&ast);
    //
    // let mut out_file = std::path::Path::new("./src").to_path_buf();
    // out_file.push("core-client.rs");
    // std::fs::write(out_file, content).unwrap();


    let src = "../../docs/openapi/core-service-storage-providers-spec.json";
    let file = std::fs::File::open(src).unwrap();
    let spec = serde_json::from_reader(file).unwrap();
    let mut generator = progenitor::Generator::default();

    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();
    let content = prettyplease::unparse(&ast);

    let mut out_file = std::path::Path::new("./src").to_path_buf();
    out_file.push("core_storage_providers.rs");
    std::fs::write(out_file, content).unwrap();


    let src = "../../docs/openapi/core-service-admin-spec.json";
    let file = std::fs::File::open(src).unwrap();
    let spec = serde_json::from_reader(file).unwrap();
    let mut generator = progenitor::Generator::default();

    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();
    let content = prettyplease::unparse(&ast);

    let mut out_file = std::path::Path::new("./src").to_path_buf();
    out_file.push("core_admin.rs");
    std::fs::write(out_file, content).unwrap();
}