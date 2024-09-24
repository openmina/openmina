use juniper::{GraphQLInputObject, GraphQLObject, GraphQLUnion};
use mina_p2p_messages::v2::MinaBaseUserCommandStableV2;
use openmina_core::transaction::Transaction;

use super::best_chain::InputGraphQLZkappCommand;

#[derive(GraphQLInputObject)]
pub struct SendZkappInput {
    pub zkapp_command: InputGraphQLZkappCommand,
}

impl From<SendZkappInput> for MinaBaseUserCommandStableV2 {
    fn from(value: SendZkappInput) -> Self {
        value.zkapp_command.into()
    }
}
