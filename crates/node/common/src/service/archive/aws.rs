use std::env;

use super::Error;

pub(crate) struct ArchiveAWSClient {
    client: aws_sdk_s3::Client,
    bucket_name: String,
    bucket_path: String,
}

impl ArchiveAWSClient {
    pub async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        let bucket_name = env::var("MINA_AWS_BUCKET_NAME")
            .map_err(|_| Error::EnvironmentVariableNotSet("MINA_AWS_BUCKET_NAME".to_string()))?;
        let bucket_path = env::var("MINA_AWS_BUCKET_PATH")
            .map_err(|_| Error::EnvironmentVariableNotSet("MINA_AWS_BUCKET_PATH".to_string()))?;
        Ok(Self {
            client: aws_sdk_s3::Client::new(&config),
            bucket_name,
            bucket_path,
        })
    }

    pub async fn upload_block(&self, key: &str, data: &[u8]) -> Result<(), Error> {
        self.client
            .put_object()
            .bucket(self.bucket_name.clone())
            .key(format!("{}/{}", self.bucket_path, key))
            .body(data.to_vec().into())
            .send()
            .await
            .map_err(|e| Error::UploadError(e.to_string()))?;

        Ok(())
    }
}
