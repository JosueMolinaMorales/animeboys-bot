use aws_sdk_ec2::Region;

pub mod ecs_commands;

#[derive(Debug)]
pub struct Ec2Error {
    pub message: String,
}

impl std::fmt::Display for Ec2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Ec2Error {
    pub fn new(message: String) -> Ec2Error {
        Ec2Error { message }
    }
}

impl std::error::Error for Ec2Error {}

pub async fn create_ecs_client() -> aws_sdk_ec2::Client {
    // Create ec2 client
    let config = aws_config::from_env()
        .region(Region::new("us-east-1"))
        .load()
        .await;
    let env_config =
        aws_config::environment::credentials::EnvironmentVariableCredentialsProvider::new();
    let ec2_config_builder = aws_sdk_ec2::config::Builder::from(&config)
        .credentials_provider(env_config)
        .build();
    let client = aws_sdk_ec2::Client::from_conf(ec2_config_builder);
    client
}
