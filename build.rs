use windres::Build;

fn main() {
    let resource_path= std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");
    let ok_icon = resource_path.clone().join("icons.rc");
    Build::new().compile(ok_icon).unwrap();
}