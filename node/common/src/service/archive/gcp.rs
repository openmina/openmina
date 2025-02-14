use gcs::http::objects::upload as gcs_upload;
use google_cloud_auth::credentials::CredentialsFile as GcpCredentialsFile;
use google_cloud_storage as gcs;

use super::Error;
use std::env;

pub(crate) struct ArchiveGCPClient {
    client: gcs::client::Client,
    bucket_name: String,
}

impl ArchiveGCPClient {
    pub async fn new() -> Result<Self, Error> {
        let get_env_var = |var: &str| -> Result<String, Error> {
            env::var(var).map_err(|_| Error::EnvironmentVariableNotSet(var.to_string()))
        };

        let cred_file = get_env_var("GCP_CREDENTIALS_JSON")?;
        let bucket_name = get_env_var("GCP_BUCKET_NAME")?;

        let credentials = GcpCredentialsFile::new_from_file(cred_file)
            .await
            .map_err(|e| Error::UploadError(format!("GCP credentials error: {}", e)))?;

        let config = gcs::client::ClientConfig::default()
            .with_credentials(credentials)
            .await
            .map_err(|e| Error::UploadError(format!("GCP config error: {}", e)))?;

        Ok(ArchiveGCPClient {
            client: gcs::client::Client::new(config),
            bucket_name,
        })
    }

    pub async fn upload_block(&self, key: &str, data: &[u8]) -> Result<(), Error> {
        let upload_type = gcs_upload::UploadType::Simple(gcs_upload::Media::new(key.to_string()));

        self.client
            .upload_object(
                &gcs_upload::UploadObjectRequest {
                    bucket: self.bucket_name.clone(),
                    ..Default::default()
                },
                data.to_vec(),
                &upload_type,
            )
            .await
            .map_err(|e| Error::UploadError(format!("GCP upload failed: {}", e)))?;

        Ok(())
    }
}
