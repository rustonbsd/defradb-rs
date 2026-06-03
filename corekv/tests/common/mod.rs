#[cfg(test)]
pub fn db_opts() -> corekv::OpenOptions {
    corekv::OpenOptions::builder().in_memory(true).build()
}