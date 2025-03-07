#[derive(Debug, Clone)]
pub struct RpcConfig {
    /// RPC bind address
    pub bind: String,
    
    /// RPC port
    pub port: String,
    
    /// RPC authentication information
    pub auth: RpcAuth,
    
    /// Allow access from these IPs
    pub allow_ips: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RpcAuth {
    pub user: Option<String>,
    pub password: Option<String>,
    pub auth_pairs: Vec<String>, // for rpcauth
}