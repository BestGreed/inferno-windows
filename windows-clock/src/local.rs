use std::io;

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn QueryPerformanceCounter(value: *mut i64) -> i32;
    fn QueryPerformanceFrequency(value: *mut i64) -> i32;
}

pub struct LocalClock {
    #[cfg(windows)]
    frequency: i64,
    #[cfg(not(windows))]
    start: std::time::Instant,
}

impl LocalClock {
    pub fn new() -> io::Result<Self> {
        #[cfg(windows)]
        {
            let mut frequency = 0;
            if unsafe { QueryPerformanceFrequency(&mut frequency) } == 0 || frequency <= 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { frequency })
        }
        #[cfg(not(windows))]
        Ok(Self {
            start: std::time::Instant::now(),
        })
    }

    pub fn now_ns(&self) -> i64 {
        #[cfg(windows)]
        {
            let mut ticks = 0;
            assert_ne!(unsafe { QueryPerformanceCounter(&mut ticks) }, 0);
            ((ticks as i128 * 1_000_000_000) / self.frequency as i128) as i64
        }
        #[cfg(not(windows))]
        {
            self.start
                .elapsed()
                .as_nanos()
                .try_into()
                .expect("local clock overflow")
        }
    }
}
