import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NetworkBlocksState } from '@network/blocks/network-blocks.state';
import { checkSorting, Sort, stateSliceAsPromise } from '../../../support/commands';
import { AppState } from '@app/app.state';

const condition = (state: NetworkBlocksState) => state && state.blocks?.length > 2;
const networkBlocksState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkBlocksState>(store, condition, 'network', 'blocks');
const isFeatureEnabled = (state: AppState) => state && state.activeNode && state.activeNode.features?.network?.includes('blocks');
const activeNode = (store: Store<MinaState>) => stateSliceAsPromise<AppState>(store, isFeatureEnabled, 'app');


describe('NETWORK BLOCKS TABLE', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network/blocks');
  });

  it('shows correct active page', () => {
    cy.window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (condition(state)) {
          cy.get('mina-toolbar .toolbar > div:first-child > span')
            .then((span: any) => expect(span.text()).equal('Network'))
            .get('mina-toolbar .submenus a.active')
            .then((a: any) => expect(a.text().trim().toLowerCase()).equals('blocks'));
        }
      });
  });

  it('displays messages in the table', () => {
    cy.window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (condition(state)) {
          expect(state.blocks.length).above(2);
          cy.get('mina-network-blocks .mina-table')
            .get('.row')
            .should('have.length.above', 2);
        }
      });
  });

  it('filter by candidate', () => {
    cy.window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (condition(state)) {
          cy.wait(1000)
            .get('mina-network-blocks-toolbar > div:nth-child(2) div button:nth-child(2)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((network: NetworkBlocksState) => {
              expect(network.filteredBlocks.every(m => m.hash === network.activeFilters[0])).to.be.true;
              cy.get('mina-network-blocks .mina-table')
                .get('.row:not(.head) > span:nth-child(3)')
                .then((rows) => {
                  Array.from(rows).forEach((row: any) => {
                    expect(row.textContent).to.includes(network.activeFilters[0].substring(0, 5));
                  });
                });
            });
        }
      });
  });

  it('have as many filters as unique candidates from the messages', () => {
    cy.window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (condition(state)) {
          const expectedCandidates = state.blocks.map(m => m.hash).filter((v, i, a) => a.indexOf(v) === i).length;
          expect(state.allFilters.length).to.equal(expectedCandidates);
        }
      });
  });

  it('sort by date', () => {
    cy.window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (condition(state)) {
          checkSorting(state.filteredBlocks, 'date', Sort.DSC);
        }
      });
  });

  it('sort by date reversed', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.mina-table .head > span:nth-child(2)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                checkSorting(state.filteredBlocks, 'date', Sort.ASC);
              }
            });
        }
      });
  });

  it('sort by hash', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.mina-table .head > span:nth-child(3)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                checkSorting(state.filteredBlocks, 'hash', Sort.DSC);
              }
            });
        }
      });
  });

  it('sort by height', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.mina-table .head > span:nth-child(4)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                checkSorting(state.filteredBlocks, 'height', Sort.DSC);
              }
            });
        }
      });
  });

  it('sort by sender', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.mina-table .head > span:nth-child(5)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                checkSorting(state.filteredBlocks, 'sender', Sort.DSC);
              }
            });
        }
      });
  });

  it('sort by receiver', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.mina-table .head > span:nth-child(6)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                checkSorting(state.filteredBlocks, 'receiver', Sort.DSC);
              }
            });
        }
      });
  });

  it('sort by recv time', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.mina-table .head > span:nth-child(7)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                checkSorting(state.filteredBlocks, 'receivedLatency', Sort.DSC);
              }
            });
        }
      });
  });

  it('sort by sent time', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.mina-table .head > span:nth-child(8)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                checkSorting(state.filteredBlocks, 'sentLatency', Sort.DSC);
              }
            });
        }
      });
  });

  it('sort by sent message kind', () => {
    cy.window()
      .its('store')
      .then(activeNode)
      .then((state: AppState) => {
        if (isFeatureEnabled(state)) {
          cy.get('.mina-table .head > span:nth-child(9)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                checkSorting(state.filteredBlocks, 'messageKind', Sort.DSC);
              }
            });
        }
      });
  });
});
