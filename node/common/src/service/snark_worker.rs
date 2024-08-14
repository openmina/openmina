use mina_p2p_messages::v2;
use node::external_snark_worker::{ExternalSnarkWorkerError, SnarkWorkSpec};

use crate::NodeService;

pub struct SnarkWorker {}

impl node::service::ExternalSnarkWorkerService for NodeService {
    fn start(
        &mut self,
        _public_key: v2::NonZeroCurvePoint,
        _fee: v2::CurrencyFeeStableV1,
    ) -> Result<(), ExternalSnarkWorkerError> {
        todo!()
    }

    fn kill(&mut self) -> Result<(), ExternalSnarkWorkerError> {
        todo!()
    }

    fn submit(&mut self, _spec: SnarkWorkSpec) -> Result<(), ExternalSnarkWorkerError> {
        todo!()
    }

    fn cancel(&mut self) -> Result<(), ExternalSnarkWorkerError> {
        todo!()
    }
}
