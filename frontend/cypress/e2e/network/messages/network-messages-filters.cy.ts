import { NetworkMessagesState } from '@network/messages/network-messages.state';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { AppState } from '@app/app.state';


const condition = (state: NetworkMessagesState) => state && state.messages?.length > 20;
const networkMessagesState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkMessagesState>(store, condition, 'network', 'messages');
const isFeatureEnabled = (state: AppState) => state && state.activeNode && state.activeNode.features?.network?.includes('messages');
const activeNode = (store: Store<MinaState>) => stateSliceAsPromise<AppState>(store, isFeatureEnabled, 'app');

describe('NETWORK MESSAGES FILTERS', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network');
  });

  it('toggle filters', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.filters-container div:nth-child(5)')
            .should('not.exist')
            .get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(5)')
            .should('exist')
            .get('.toggle-filters')
            .click()
            .wait(600)
            .get('.filters-container div:nth-child(5)')
            .should('exist')
            .should('not.be.visible');
        }
      });
  });

  it('filter messages by multistream', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(1) .fx-row-vert-cent:nth-child(1) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', 'multistream');
                cy.url().should('include', 'stream_kind');
                cy.get('.filter-row .flex-wrap button').should('have.length', 1);
                expect(state.messages.every(message => message.streamKind === '/multistream/1.0.0')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by kad', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(2) .fx-row-vert-cent:nth-child(2) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', 'put_value,get_value,add_provider,get_providers,find_node,ping');
                cy.get('.filter-row .flex-wrap button').should('have.length', 6);
                expect(state.messages.every(message => message.streamKind === '/coda/kad/1.0.0')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by peer exchange', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(3) .fx-row-vert-cent:nth-child(1) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', encodeURIComponent('/mina/peer-exchange'));
                cy.get('.filter-row .flex-wrap button').should('have.length', 1);
                expect(state.messages.every(message => message.streamKind === '/mina/peer-exchange')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by coda mplex', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(1) .fx-row-vert-cent:nth-child(2) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', encodeURIComponent('/coda/mplex/1.0.0'));
                cy.get('.filter-row .flex-wrap button').should('have.length', 1);
                expect(state.messages.every(message => message.streamKind === '/coda/mplex/1.0.0')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by identify', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(4) .fx-row-vert-cent:nth-child(1) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', encodeURIComponent('/ipfs/id/1.0.0'));
                cy.get('.filter-row .flex-wrap button').should('have.length', 1);
                expect(state.messages.every(message => message.streamKind === '/ipfs/id/1.0.0')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by IPFS push', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(4) .fx-row-vert-cent:nth-child(2) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', encodeURIComponent('/ipfs/id/push/1.0.0'));
                cy.get('.filter-row .flex-wrap button').should('have.length', 1);
                expect(state.messages.every(message => message.streamKind === '/ipfs/id/push/1.0.0')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by meshsub', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(5) .fx-row-vert-cent:nth-child(1) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', 'subscribe,unsubscribe,meshsub_iwant,meshsub_ihave,meshsub_prune,meshsub_graft,publish_new_state,publish_snark_pool_diff,publish_transaction_pool_diff');
                cy.get('.filter-row .flex-wrap button').should('have.length', 9);
                expect(state.messages.every(message => message.streamKind === '/meshsub/1.1.0')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by rpcs', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(6) .fx-row-vert-cent:nth-child(1) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', 'get_some_initial_peers,get_staged_ledger_aux_and_pending_coinbases_at_hash,answer_sync_ledger_query,get_ancestry,get_best_tip,get_node_status,get_transition_chain_proof,get_transition_chain,get_transition_knowledge,get_epoch_ledger,__Versioned_rpc.Menu,ban_notify');
                cy.get('.filter-row .flex-wrap button').should('have.length', 12);
                expect(state.messages.every(message => message.streamKind === 'coda/rpcs/0.0.1')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by unknown', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(7) .fx-row-vert-cent:nth-child(1) .category')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', encodeURIComponent('unknown'));
                cy.get('.filter-row .flex-wrap button').should('have.length', 1);
                expect(state.messages.every(message => message.streamKind === 'unknown')).to.be.true;
              }
            });
        }
      });
  });

  // failing - broken backend
  it.skip('filter messages by subscribe and publish snark pool diff', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(5) .fx-row-vert-cent:nth-child(1) .filter:nth-child(2)')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(5) .fx-row-vert-cent:nth-child(1) .filter:nth-child(9)')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', 'subscribe');
                cy.url().should('include', 'publish_snark_pool_diff');
                cy.get('.filter-row .flex-wrap button').should('have.length', 2);
                expect(state.messages.every(message => message.messageKind === 'subscribe' || message.messageKind === 'publish_snark_pool_diff')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by get epoch ledger and versioned rpc menu', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(6) .fx-row-vert-cent:nth-child(1) .filter:nth-child(11)')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(6) .fx-row-vert-cent:nth-child(1) .filter:nth-child(12)')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', 'get_epoch_ledger');
                cy.url().should('include', encodeURIComponent('__Versioned_rpc.Menu'));
                cy.get('.filter-row .flex-wrap button').should('have.length', 2);
                expect(state.messages.every(message => message.messageKind.includes('get_epoch_ledger') || message.messageKind.includes('__Versioned_rpc.Menu'))).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by find node and get ancestry', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(2) .fx-row-vert-cent:nth-child(2) .filter:nth-child(6)')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(6) .fx-row-vert-cent:nth-child(1) .filter:nth-child(5)')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', 'get_ancestry');
                cy.url().should('include', 'find_node');
                cy.get('.filter-row .flex-wrap button').should('have.length', 2);
                expect(state.messages.every(message => message.messageKind.includes('get_ancestry') || message.messageKind === 'find_node')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by transaction pool diff and answer sync ledger query', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(5) .fx-row-vert-cent:nth-child(1) .filter:nth-child(10)')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(6) .fx-row-vert-cent:nth-child(1) .filter:nth-child(4)')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                cy.url().should('include', 'answer_sync_ledger_query');
                cy.url().should('include', 'publish_transaction_pool_diff');
                cy.get('.filter-row .flex-wrap button').should('have.length', 2);
                expect(state.messages.every(message => message.messageKind.includes('answer_sync_ledger_query') || message.messageKind === 'publish_transaction_pool_diff')).to.be.true;
              }
            });
        }
      });
  });

  it('filter messages by coda yamux and delete the filter', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.toggle-filters')
            .click()
            .get('.filters-container div:nth-child(2) > .flex-row:nth-child(1) .fx-row-vert-cent:nth-child(3) .filter:nth-child(2)')
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                expect(state.messages.every(message => message.messageKind === 'yamux')).to.be.true;
                cy.url().should('include', 'yamux')
                  .url().should('include', 'coda')
                  .get('.filter-row .flex-wrap button').should('have.length', 1)
                  .get('.filter-row .flex-wrap button:nth-child(1)')
                  .click()
                  .get('.filter-row .flex-wrap button').should('have.length', 0)
                  .url().should('not.include', 'yamux')
                  .url().should('not.include', 'coda')
                  .wait(1500)
                  .window()
                  .its('store')
                  .then(networkMessagesState)
                  .then((network: NetworkMessagesState) => {
                    if (condition(state)) {
                      expect(network.messages.every(message => message.messageKind === 'yamux')).to.be.false;
                    }
                  });
              }
            });
        }
      });
  });
});
