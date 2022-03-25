pub fn set_num_threads(count: usize) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(count)
        .build_global()
        .unwrap();
}
