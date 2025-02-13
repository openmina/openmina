use bitflags::bitflags;
use std::env;

bitflags! {
    #[derive(Debug, Clone, Default)]
    pub struct ArchiveStorageOptions: u8 {
        const ARCHIVER_PROCESS = 0b0001;
        const LOCAL_PRECOMPUTED_STORAGE = 0b0010;
        const GCP_PRECOMPUTED_STORAGE = 0b0100;
        const AWS_PRECOMPUTED_STORAGE = 0b1000;
    }
}

impl ArchiveStorageOptions {
    pub fn is_enabled(&self) -> bool {
        !self.is_empty()
    }

    pub fn requires_precomputed_block(&self) -> bool {
        self.uses_aws_precomputed_storage()
            || self.uses_gcp_precomputed_storage()
            || self.uses_local_precomputed_storage()
    }

    pub fn validate_env_vars(&self) -> Result<(), String> {
        if self.contains(ArchiveStorageOptions::ARCHIVER_PROCESS)
            && env::var("OPENMINA_ARCHIVE_ADDRESS").is_err()
        {
            return Err(
                "OPENMINA_ARCHIVE_ADDRESS is required when ARCHIVER_PROCESS is enabled".to_string(),
            );
        }

        if self.uses_aws_precomputed_storage() {
            if env::var("AWS_ACCESS_KEY_ID").is_err() {
                return Err(
                    "AWS_ACCESS_KEY_ID is required when AWS_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }
            if env::var("AWS_SECRET_ACCESS_KEY").is_err() {
                return Err(
                    "AWS_SECRET_ACCESS_KEY is required when AWS_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }
            if env::var("AWS_SESSION_TOKEN").is_err() {
                return Err(
                    "AWS_SESSION_TOKEN is required when AWS_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }

            if env::var("AWS_DEFAULT_REGION").is_err() {
                return Err(
                    "AWS_DEFAULT_REGION is required when AWS_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }

            if env::var("OPENMINA_AWS_BUCKET_NAME").is_err() {
                return Err(
                    "OPENMINA_AWS_BUCKET_NAME is required when AWS_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }

            // if env::var("OPENMINA_AWS_BUCKET_PATH").is_err() {
            //     return Err(
            //         "OPENMINA_AWS_BUCKET_PATH is required when AWS_PRECOMPUTED_STORAGE is enabled"
            //             .to_string(),
            //     );
            // }
        }

        if self.uses_gcp_precomputed_storage() {
            if env::var("GCP_CREDENTIALS_JSON").is_err() {
                return Err(
                    "GCP_CREDENTIALS_JSON is required when GCP_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }

            if env::var("GCP_BUCKET_NAME").is_err() {
                return Err(
                    "GCP_BUCKET_NAME is required when GCP_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }
        }

        Ok(())
    }

    pub fn uses_local_precomputed_storage(&self) -> bool {
        self.contains(ArchiveStorageOptions::LOCAL_PRECOMPUTED_STORAGE)
    }

    pub fn uses_archiver_process(&self) -> bool {
        self.contains(ArchiveStorageOptions::ARCHIVER_PROCESS)
    }

    pub fn uses_gcp_precomputed_storage(&self) -> bool {
        self.contains(ArchiveStorageOptions::GCP_PRECOMPUTED_STORAGE)
    }

    pub fn uses_aws_precomputed_storage(&self) -> bool {
        self.contains(ArchiveStorageOptions::AWS_PRECOMPUTED_STORAGE)
    }
}
