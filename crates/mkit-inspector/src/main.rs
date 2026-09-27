fn main() {
    if let Err(error) = mkit_inspector::serve_stdio(mkit_inspector::GpuiInspector::default()) {
        eprintln!("mkit-inspector: {error}");
        std::process::exit(1);
    }
}
