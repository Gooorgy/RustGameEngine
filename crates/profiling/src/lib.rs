#[cfg(feature = "profiling")]
pub use tracy_client;

// wrapped return type. keep alive for the duration of the program.
#[cfg(feature = "profiling")]
pub struct ProfilerGuard(#[allow(dead_code)] tracy_client::Client);
#[cfg(not(feature = "profiling"))]
pub struct ProfilerGuard;

#[cfg(feature = "profiling")]
pub fn start() -> ProfilerGuard {
    ProfilerGuard(tracy_client::Client::start())
}
#[cfg(not(feature = "profiling"))]
pub fn start() -> ProfilerGuard {
    ProfilerGuard
}

#[cfg(feature = "profiling")]
#[inline(always)]
pub fn frame_mark() {
    tracy_client::frame_mark();
}
#[cfg(not(feature = "profiling"))]
#[inline(always)]
pub fn frame_mark() {}


#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! scope {
    ($name:literal) => {
        let _prof_span = $crate::tracy_client::span!($name);
    };
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! scope {
    ($name:literal) => {};
}

#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! set_thread_name {
    ($name:literal) => {
        $crate::tracy_client::set_thread_name!($name)
    };
}
#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! set_thread_name {
    ($name:literal) => {};
}

#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! plot {
    ($name:literal, $value:expr) => {
        $crate::tracy_client::plot!($name, $value as f64)
    };
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! plot {
    ($name:literal, $value:expr) => {};
}