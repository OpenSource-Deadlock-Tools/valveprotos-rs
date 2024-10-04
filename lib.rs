pub mod common {
    include!(concat!(env!("OUT_DIR"), "/common.rs"));
}

pub mod gcsdk {
    include!(concat!(env!("OUT_DIR"), "/gcsdk.rs"));
}

#[cfg(feature = "deadlock")]
pub mod deadlock {
    include!(concat!(env!("OUT_DIR"), "/deadlock.rs"));
}

#[cfg(feature = "dota2")]
pub mod dota2 {
    include!(concat!(env!("OUT_DIR"), "/dota2.rs"));
}
