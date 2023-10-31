use revm_primitives::U256;

/// Configuration of a single account
pub struct AccountConfig {
    /// the secret key of the account
    pub secret_key: k256::SecretKey,
    /// the balance of the account
    pub balance: U256,
}
