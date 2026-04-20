//! gRPC services for BeeBotOS Gateway

pub mod skills;

// Generated proto modules matching prost's expected hierarchy
pub mod generated {
    pub mod beebotos {
        pub mod common {
            include!(concat!(env!("OUT_DIR"), "/beebotos.common.rs"));
        }
        pub mod skills {
            pub mod registry {
                include!(concat!(env!("OUT_DIR"), "/beebotos.skills.registry.rs"));
            }
        }
    }
}
