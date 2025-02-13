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
        let cred_file = env::var("GCP_CREDENTIALS_JSON")
            .map_err(|_| Error::EnvironmentVariableNotSet("GCP_CREDENTIALS_JSON".to_string()))?;
        let bucket_name = env::var("GCP_BUCKET_NAME")
            .map_err(|_| Error::EnvironmentVariableNotSet("GCP_BUCKET_NAME".to_string()))?;
        let credentials = GcpCredentialsFile::new_from_file(cred_file).await.unwrap();
        let config = gcs::client::ClientConfig::default()
            .with_credentials(credentials)
            .await
            .unwrap();
        let client = gcs::client::Client::new(config);
        Ok(ArchiveGCPClient {
            client,
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
            .map_err(|e| Error::UploadError(e.to_string()))?;
        Ok(())
    }
}
