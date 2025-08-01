#![allow(unused_macros)]

#[cfg(feature = "log")]
macro_rules! error {
    ($($t:tt)*) => {{ log::error!($($t)*); }};
}

#[cfg(feature = "log")]
macro_rules! warn {
    ($($t:tt)*) => {{ log::warn!($($t)*); }};
}

#[cfg(feature = "log")]
macro_rules! info {
    ($($t:tt)*) => {{ log::info!($($t)*); }};
}

#[cfg(feature = "log")]
macro_rules! debug {
    ($($t:tt)*) => {{ log::debug!($($t)*); }};
}

#[cfg(feature = "log")]
macro_rules! trace {
    ($($t:tt)*) => {{ log::trace!($($t)*); }};
}

#[cfg(feature = "defmt")]
macro_rules! error {
    ($($t:tt)*) => {{ defmt::error!($($t)*); }};
}

#[cfg(feature = "defmt")]
macro_rules! warn {
    ($($t:tt)*) => {{ defmt::warn!($($t)*); }};
}

#[cfg(feature = "defmt")]
macro_rules! info {
    ($($t:tt)*) => {{ defmt::info!($($t)*); }};
}

#[cfg(feature = "defmt")]
macro_rules! debug {
    ($($t:tt)*) => {{ defmt::debug!($($t)*); }};
}

#[cfg(feature = "defmt")]
macro_rules! trace {
    ($($t:tt)*) => {{ defmt::trace!($($t)*); }};
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
macro_rules! error {
    ($($t:tt)*) => {{ format_args!($($t)*); }};
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
macro_rules! warn {
    ($($t:tt)*) => {{ format_args!($($t)*); }};
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
macro_rules! info {
    ($($t:tt)*) => {{ format_args!($($t)*); }};
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
macro_rules! debug {
    ($($t:tt)*) => {{ format_args!($($t)*); }};
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
macro_rules! trace {
    ($($t:tt)*) => {{ format_args!($($t)*); }};
}
