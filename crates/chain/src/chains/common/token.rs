//! Token Utilities
//!
//! Provides standardized token formatting and parsing for all EVM chains.
//! Supports any decimal precision (default: 18 for ETH-compatible chains).

use alloy_primitives::U256;

/// Standard ERC20 token decimals
pub const DEFAULT_TOKEN_DECIMALS: u8 = 18;

/// Wei per ETH (10^18)
pub const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000;

/// Format token amount with specified decimals
///
/// # Examples
/// ```
/// use alloy_primitives::U256;
/// use beebotos_chain::chains::common::token::format_token_amount;
///
/// let wei = U256::from(1_500_000_000_000_000_000u64);
/// let eth = format_token_amount(wei, 18, "ETH");
/// assert!(eth.starts_with("1.5"));
/// ```
pub fn format_token_amount(amount: U256, decimals: u8, symbol: &str) -> String {
    if decimals == 0 {
        return format!("{} {}", amount, symbol);
    }

    let divisor = U256::from(10).pow(U256::from(decimals));
    let whole = amount / divisor;
    let remainder = amount % divisor;

    let frac_str = format!("{:0>width$}", remainder, width = decimals as usize);
    let frac_trimmed = frac_str.trim_end_matches('0');

    if frac_trimmed.is_empty() {
        format!("{} {}", whole, symbol)
    } else {
        format!(
            "{}.{:0>width$} {}",
            whole,
            remainder,
            symbol,
            width = decimals as usize
        )
    }
}

/// Format native token amount (18 decimals) without symbol
///
/// Internal helper for common 18-decimal formatting
pub fn format_native_amount(amount: U256) -> String {
    let divisor = U256::from(WEI_PER_ETH);
    let whole = amount / divisor;
    let remainder = amount % divisor;
    format!("{}.{:018}", whole, remainder)
}

/// Parse token amount string to U256
///
/// # Examples
/// ```
/// use beebotos_chain::chains::common::token::parse_token_amount;
///
/// let wei = parse_token_amount("1.5", 18).unwrap();
/// assert_eq!(wei.to_string(), "1500000000000000000");
/// ```
pub fn parse_token_amount(amount: &str, decimals: u8) -> Option<U256> {
    let parts: Vec<&str> = amount.split('.').collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }

    let whole: u64 = parts[0].parse().ok()?;
    let whole = U256::from(whole);

    if parts.len() == 2 {
        let frac_str = parts[1];
        let frac_val: u64 = frac_str.parse().ok()?;
        let frac = U256::from(frac_val);
        // Use provided decimals, not the length of fraction string
        let multiplier = U256::from(10).pow(U256::from(decimals));
        let frac_multiplier = U256::from(10).pow(U256::from(frac_str.len()));
        let adjusted_frac = frac * multiplier / frac_multiplier;
        return Some(whole * multiplier + adjusted_frac);
    }

    Some(whole)
}

/// Parse native token (18 decimals) string to U256
///
/// # Examples
/// ```
/// use beebotos_chain::chains::common::token::parse_native_amount;
///
/// let wei = parse_native_amount("1.5").unwrap();
/// assert_eq!(wei.to_string(), "1500000000000000000");
/// ```
pub fn parse_native_amount(amount: &str) -> Option<U256> {
    let parts: Vec<&str> = amount.split('.').collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }

    let whole: u64 = parts[0].parse().ok()?;
    let whole = U256::from(whole);
    let mut frac = U256::ZERO;

    if parts.len() == 2 {
        let frac_str = format!("{:0<18}", parts[1]);
        let frac_str = &frac_str[..18.min(frac_str.len())];
        let frac_val: u64 = frac_str.parse().ok()?;
        frac = U256::from(frac_val);
    }

    let multiplier = U256::from(WEI_PER_ETH);
    Some(whole * multiplier + frac)
}

/// Chain-specific token formatters for convenience
pub mod chain_formatters {
    use super::*;

    /// Format ETH amount (18 decimals)
    pub fn format_eth(wei: U256) -> String {
        format_token_amount(wei, 18, "ETH")
    }

    /// Parse ETH string to wei
    pub fn parse_eth(eth: &str) -> Option<U256> {
        parse_native_amount(eth)
    }

    /// Format BNB amount (18 decimals)
    pub fn format_bnb(wei: U256) -> String {
        format_token_amount(wei, 18, "BNB")
    }

    /// Parse BNB string to wei
    pub fn parse_bnb(bnb: &str) -> Option<U256> {
        parse_native_amount(bnb)
    }

    /// Format BKC (Beechain) amount (18 decimals)
    pub fn format_bkc(wei: U256) -> String {
        format_token_amount(wei, 18, "BKC")
    }

    /// Parse BKC string to wei
    pub fn parse_bkc(bkc: &str) -> Option<U256> {
        parse_native_amount(bkc)
    }

    /// Format MONAD amount (18 decimals)
    pub fn format_monad(wei: U256) -> String {
        format_token_amount(wei, 18, "MON")
    }

    /// Parse MONAD string to wei
    pub fn parse_monad(monad: &str) -> Option<U256> {
        parse_native_amount(monad)
    }
}

/// Transaction priority levels with gas price multipliers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransactionPriority {
    Low,    // Standard gas price
    Normal, // +10%
    High,   // +20%
    Urgent, // +50%
    Custom { multiplier: f64 },
}

impl Eq for TransactionPriority {}

impl std::hash::Hash for TransactionPriority {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash only the discriminant, not the f64 value
        // This allows TransactionPriority to be used as HashMap key
        match self {
            TransactionPriority::Low => 0.hash(state),
            TransactionPriority::Normal => 1.hash(state),
            TransactionPriority::High => 2.hash(state),
            TransactionPriority::Urgent => 3.hash(state),
            TransactionPriority::Custom { .. } => 4.hash(state),
        }
    }
}

impl TransactionPriority {
    /// Get gas price multiplier
    pub fn multiplier(&self) -> f64 {
        match self {
            TransactionPriority::Low => 0.9,
            TransactionPriority::Normal => 1.0,
            TransactionPriority::High => 1.2,
            TransactionPriority::Urgent => 1.5,
            TransactionPriority::Custom { multiplier } => *multiplier,
        }
    }

    /// Get priority fee in gwei
    pub fn priority_fee_gwei(&self) -> u64 {
        match self {
            TransactionPriority::Low => 1,
            TransactionPriority::Normal => 2,
            TransactionPriority::High => 5,
            TransactionPriority::Urgent => 10,
            TransactionPriority::Custom { .. } => 2,
        }
    }

    /// Apply priority to gas price
    pub fn apply_to_gas_price(&self, base_price: u128) -> u128 {
        (base_price as f64 * self.multiplier()) as u128
    }

    /// Get priority level as a numeric value (for ordering)
    /// Higher value = higher priority
    pub fn priority_level(&self) -> u8 {
        match self {
            TransactionPriority::Low => 1,
            TransactionPriority::Normal => 2,
            TransactionPriority::High => 3,
            TransactionPriority::Urgent => 4,
            TransactionPriority::Custom { multiplier } => {
                // Map custom multiplier to priority level
                if *multiplier >= 1.5 {
                    4
                } else if *multiplier >= 1.2 {
                    3
                } else if *multiplier >= 1.0 {
                    2
                } else {
                    1
                }
            }
        }
    }

    /// Get estimated confirmation time in seconds
    pub fn confirmation_time_secs(&self) -> u64 {
        match self {
            TransactionPriority::Low => 60,    // ~1 minute
            TransactionPriority::Normal => 30, // ~30 seconds
            TransactionPriority::High => 15,   // ~15 seconds
            TransactionPriority::Urgent => 5,  // ~5 seconds
            TransactionPriority::Custom { multiplier } => {
                // Estimate based on multiplier
                if *multiplier >= 1.5 {
                    5
                } else if *multiplier >= 1.2 {
                    15
                } else if *multiplier >= 1.0 {
                    30
                } else {
                    60
                }
            }
        }
    }
}

/// Ethereum-specific priority levels (for backward compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EthereumPriority {
    Slow,
    Standard,
    Fast,
    Urgent,
    Custom { max_fee: u64, priority_fee: u64 },
}

impl From<EthereumPriority> for TransactionPriority {
    fn from(priority: EthereumPriority) -> Self {
        match priority {
            EthereumPriority::Slow => TransactionPriority::Low,
            EthereumPriority::Standard => TransactionPriority::Normal,
            EthereumPriority::Fast => TransactionPriority::High,
            EthereumPriority::Urgent => TransactionPriority::Urgent,
            EthereumPriority::Custom {
                max_fee: _,
                priority_fee,
            } => TransactionPriority::Custom {
                multiplier: priority_fee as f64 / 100.0,
            },
        }
    }
}

impl EthereumPriority {
    /// Get priority fee in gwei (for backward compatibility)
    pub fn priority_fee_gwei(&self) -> u64 {
        match self {
            EthereumPriority::Slow => 0,
            EthereumPriority::Standard => 1,
            EthereumPriority::Fast => 2,
            EthereumPriority::Urgent => 5,
            EthereumPriority::Custom { priority_fee, .. } => *priority_fee,
        }
    }

    /// Get max fee multiplier
    pub fn max_fee_multiplier(&self) -> f64 {
        match self {
            EthereumPriority::Slow => 1.0,
            EthereumPriority::Standard => 1.1,
            EthereumPriority::Fast => 1.2,
            EthereumPriority::Urgent => 1.5,
            EthereumPriority::Custom { .. } => 1.0,
        }
    }
}

/// BSC-specific priority levels (for backward compatibility)
pub type BscPriority = TransactionPriority;

/// Beechain-specific priority levels (for backward compatibility)
pub type BeechainPriority = TransactionPriority;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_token_amount() {
        let wei = U256::from(1_500_000_000_000_000_000u64);
        let result = format_token_amount(wei, 18, "TEST");
        assert!(result.contains("1.5"));
        assert!(result.contains("TEST"));
    }

    #[test]
    fn test_parse_token_amount() {
        let amount = "1.5";
        let wei = parse_token_amount(amount, 18).unwrap();
        let expected = U256::from(1_500_000_000_000_000_000u64);
        assert_eq!(wei, expected);
    }

    #[test]
    fn test_parse_native_amount() {
        let wei = parse_native_amount("1.5").unwrap();
        let expected = U256::from(1_500_000_000_000_000_000u64);
        assert_eq!(wei, expected);
    }

    #[test]
    fn test_chain_formatters() {
        let wei = U256::from(1_000_000_000_000_000_000u64);

        assert!(chain_formatters::format_eth(wei).contains("ETH"));
        assert!(chain_formatters::format_bnb(wei).contains("BNB"));
        assert!(chain_formatters::format_bkc(wei).contains("BKC"));
        assert!(chain_formatters::format_monad(wei).contains("MON"));
    }

    #[test]
    fn test_transaction_priority() {
        assert_eq!(TransactionPriority::Low.multiplier(), 0.9);
        assert_eq!(TransactionPriority::Normal.multiplier(), 1.0);
        assert_eq!(TransactionPriority::High.multiplier(), 1.2);
        assert_eq!(TransactionPriority::Urgent.multiplier(), 1.5);

        let base_price = 10_000_000_000u128; // 10 gwei
        assert_eq!(
            TransactionPriority::High.apply_to_gas_price(base_price),
            12_000_000_000
        );
    }

    #[test]
    fn test_ethereum_priority_backward_compat() {
        let eth_priority = EthereumPriority::Fast;
        let priority: TransactionPriority = eth_priority.into();
        assert_eq!(priority, TransactionPriority::High);

        assert_eq!(eth_priority.priority_fee_gwei(), 2);
        assert_eq!(eth_priority.max_fee_multiplier(), 1.2);
    }
}
