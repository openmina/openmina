use super::outgoing::P2pConnectionOutgoingInitOpts;

pub trait P2pConnectionService: redux::Service {
    fn outgoing_init(&mut self, opts: P2pConnectionOutgoingInitOpts);
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts;
}
