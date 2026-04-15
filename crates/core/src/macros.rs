//! Useful macros for BeeBotOS

/// Log macro that includes agent context
#[macro_export]
macro_rules! agent_log {
    ($agent_id:expr, $level:expr, $($arg:tt)*) => {
        log::log!(
            $level,
            "[Agent {}] {}",
            $agent_id,
            format!($($arg)*)
        )
    };
}

/// Measure execution time of a block
#[macro_export]
macro_rules! timed {
    ($name:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let elapsed = start.elapsed();
        log::debug!("{} took {:?}", $name, elapsed);
        result
    }};
}

/// Create a syscall handler match arm
#[macro_export]
macro_rules! syscall_handler {
    ($number:expr, $handler:expr) => {
        $number => $handler(args, caller).await
    };
}

/// Ensure capability or return error
#[macro_export]
macro_rules! require_capability {
    ($caps:expr, $level:expr) => {
        if let Err(e) = $caps.verify($level) {
            return Err(e.into());
        }
    };
}

/// Create agent ID from string
#[macro_export]
macro_rules! agent_id {
    ($hex:expr) => {
        AgentId::from_hex($hex).expect("Invalid agent ID")
    };
}

/// Lazy static initialization
#[macro_export]
macro_rules! lazy_static {
    ($name:ident: $type:ty = $init:expr) => {
        static $name: std::sync::LazyLock<$type> = std::sync::LazyLock::new(|| $init);
    };
}

/// Create a capability set with specific permissions
#[macro_export]
macro_rules! capability_set {
    ($level:expr, [$($perm:expr),*]) => {{
        let mut caps = CapabilitySet::with_level($level);
        $(
            caps = caps.with_permission($perm);
        )*
        caps
    }};
}

/// Test helper macro
#[cfg(test)]
#[macro_export]
macro_rules! assert_capability {
    ($caps:expr, $level:expr) => {
        assert!($caps.has($level), "Expected capability {:?}", $level)
    };
}

/// Declare system call numbers
#[macro_export]
macro_rules! declare_syscalls {
    (
        $vis:vis enum $name:ident {
            $(
                $variant:ident = $num:expr
            ),*$(,)?
        }
    ) => {
        #[repr(u64)]
        $vis enum $name {
            $(
                $variant = $num
            ),*
        }

        impl $name {
            $vis fn from_u64(n: u64) -> Option<Self> {
                match n {
                    $(
                        $num => Some(Self::$variant),
                    )*
                    _ => None,
                }
            }

            $vis fn as_u64(&self) -> u64 {
                *self as u64
            }
        }
    };
}
