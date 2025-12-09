use serde::{Deserialize, Serialize};

use super::{dtls_fingerprint::*, dtls_role::*};

/// DTLSParameters holds information relating to DTLS configuration.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DTLSParameters {
    pub role: DTLSRole,
    pub fingerprints: Vec<RTCDtlsFingerprint>,
}
