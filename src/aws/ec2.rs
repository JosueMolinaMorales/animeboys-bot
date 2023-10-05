use aws_sdk_ec2::Region;
use serenity::prelude::TypeMapKey;

use super::error::Ec2Error;

pub struct Ec2Client {
    pub client: aws_sdk_ec2::Client,
    instance_id: String,
}

impl TypeMapKey for Ec2Client {
    type Value = Ec2Client;
}

impl Ec2Client {
    pub async fn new(instance_id: String) -> Self {
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
        Self {
            client,
            instance_id,
        }
    }

    pub async fn start_instance(&self) -> Result<String, Ec2Error> {
        let res = self
            .client
            .start_instances()
            .instance_ids(self.instance_id.clone())
            .send()
            .await
            .map_err(|e| Ec2Error::new(e.to_string()))?;
        Ok(res.starting_instances().unwrap()[0]
            .current_state()
            .unwrap()
            .name()
            .unwrap()
            .as_str()
            .to_string())
    }

    pub async fn stop_instance(&self) -> Result<(), Ec2Error> {
        self.client
            .stop_instances()
            .instance_ids(self.instance_id.clone())
            .send()
            .await
            .map_err(|e| Ec2Error::new(e.to_string()))?;
        Ok(())
    }

    pub async fn get_instance_status(&self) -> Result<String, Ec2Error> {
        let res = self
            .client
            .describe_instances()
            .instance_ids(self.instance_id.clone())
            .send()
            .await
            .map_err(|e| Ec2Error::new(e.to_string()))?;
        let status = res
            .reservations()
            .ok_or(Ec2Error::new("No instances found".into()))?[0]
            .instances()
            .ok_or(Ec2Error::new("No instances found".into()))?[0]
            .state()
            .ok_or(Ec2Error::new("No state found".into()))?
            .name()
            .ok_or(Ec2Error::new("No name found".into()))?
            .as_str()
            .to_string();
        Ok(status)
    }

    pub async fn get_instance_ip(&self) -> Result<String, Ec2Error> {
        let status = self.get_instance_status().await?;
        if status != "running" {
            return Err(Ec2Error::new("Instance is not running".into()));
        }
        Ok(self
            .client
            .describe_instances()
            .instance_ids(self.instance_id.clone())
            .send()
            .await
            .map_err(|e| Ec2Error::new(e.to_string()))?
            .reservations()
            .ok_or(Ec2Error::new("No reservations found".into()))?[0]
            .instances()
            .ok_or(Ec2Error::new("No instances found".into()))?[0]
            .public_ip_address()
            .ok_or(Ec2Error::new("No public ip found".into()))?
            .to_string())
    }
}
