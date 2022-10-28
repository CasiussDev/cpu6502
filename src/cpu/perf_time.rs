use std::time;

#[derive(Copy, Clone, Debug, Default)]
pub struct PerfTimer {
    pub executing: time::Duration,
    pub decoding: time::Duration,
    pub running: time::Duration,
}
