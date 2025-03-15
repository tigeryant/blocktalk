#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub bind: String,
    pub port: String,
    pub auth: RpcAuth,
    pub allow_ips: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RpcAuth {
    pub user: Option<String>,
    pub password: Option<String>,
    pub auth_pairs: Vec<String>,
}
