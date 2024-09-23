use juniper::{GraphQLInputObject, GraphQLObject, GraphQLUnion};

use super::best_chain::InputGraphQLZkappCommand;



#[derive(GraphQLInputObject)]
pub struct SendZkappInput {
    pub zkapp_command: InputGraphQLZkappCommand,
}