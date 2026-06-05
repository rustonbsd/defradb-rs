use corekv::OpenOptions;

#[allow(dead_code)]
pub fn db_opts() -> OpenOptions {
    OpenOptions::builder().in_memory(true).build()
}
