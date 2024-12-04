use serde::{Deserialize, Serialize};

use crate::node::OcamlNodeExecutable;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClusterConfig {
    #[serde(default)]
    port_range: Option<(u16, u16)>,
    all_rust_to_rust_use_webrtc: bool,
    proof_kind: ProofKind,
    #[serde(default)]
    is_replay: bool,
    #[serde(default)]
    use_debugger: bool,
    #[serde(default)]
    ocaml_node_executable: Option<OcamlNodeExecutable>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ProofKind {
    Dummy,
    ConstraintsChecked,
    Full,
}

impl Default for ProofKind {
    fn default() -> Self {
        // once it's working, change to Self::ConstraintsChecked
        Self::Dummy
    }
}

impl ClusterConfig {
    pub fn new(ocaml_node_executable: Option<OcamlNodeExecutable>) -> anyhow::Result<Self> {
        Ok(Self {
            port_range: None,
            all_rust_to_rust_use_webrtc: false,
            proof_kind: ProofKind::default(),
            is_replay: false,
            use_debugger: false,
            ocaml_node_executable,
        })
    }

    pub fn use_debugger(&mut self) -> &mut Self {
        self.use_debugger = true;
        self
    }

    pub fn is_use_debugger(&self) -> bool {
        self.use_debugger
    }

    pub fn set_replay(&mut self) -> &mut Self {
        self.is_replay = true;
        self
    }

    pub fn is_replay(&self) -> bool {
        self.is_replay
    }

    pub fn port_range(&self) -> std::ops::RangeInclusive<u16> {
        let range = self.port_range.unwrap_or((11_000, 49_151));
        (range.0)..=(range.1)
    }

    pub fn set_all_rust_to_rust_use_webrtc(&mut self) -> &mut Self {
        assert!(cfg!(feature = "p2p-webrtc"));
        self.all_rust_to_rust_use_webrtc = true;
        self
    }

    pub fn all_rust_to_rust_use_webrtc(&self) -> bool {
        self.all_rust_to_rust_use_webrtc
    }

    pub fn set_proof_kind(&mut self, kind: ProofKind) -> &mut Self {
        self.proof_kind = kind;
        self
    }

    pub fn proof_kind(&self) -> ProofKind {
        self.proof_kind
    }

    pub fn set_ocaml_node_executable(&mut self, executable: OcamlNodeExecutable) -> &mut Self {
        self.ocaml_node_executable = Some(executable);
        self
    }

    pub fn ocaml_node_executable(&mut self) -> OcamlNodeExecutable {
        self.ocaml_node_executable
            .get_or_insert_with(|| {
                OcamlNodeExecutable::find_working().expect("cannot run OCaml node")
            })
            .clone()
    }
}
