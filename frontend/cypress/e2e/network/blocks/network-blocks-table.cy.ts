import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NetworkBlocksState } from '@network/blocks/network-blocks.state';
import { stateSliceAsPromise } from '../../../support/commands';

const condition = (state: NetworkBlocksState) => state && state.blocks.length > 2;
const networkBlocksState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkBlocksState>(store, condition, 'network', 'blocks');


describe('NETWORK BLOCKS TABLE', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network/blocks');
  });

  it('shows correct active page', () => {
    cy.get('mina-toolbar .toolbar > div:first-child > span')
      .then((span: any) => expect(span.text()).equal('Network'))
      .get('mina-submenu-tabs a.active')
      .then((a: any) => expect(a.text().toLowerCase()).equals('blocks'));
  });

  it('displays messages in the table', () => {
    cy.window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
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
        if (state && state.allFilters.length > 0) {
          cy.wait(1000)
            .get('mina-network-blocks-toolbar > div:nth-child(2) button:nth-child(2)')
            .click()
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((network: NetworkBlocksState) => {
              expect(network.filteredBlocks.every(m => m.hash === network.activeFilters[0])).to.be.true;
              cy.get('mina-network-blocks .mina-table')
                .get('.row:not(.head) > span:nth-child(3)')
                .then((rows: any[]) => {
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
        if (state && state.blocks.length > 2) {
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
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].date || '';
            const next = state.filteredBlocks[i + 1].date || '';
            if (next.localeCompare(curr) < 0) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });

  it('sort by date reversed', () => {
    cy.get('.mina-table .head > span:nth-child(2)')
      .click()
      .window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].date || '';
            const next = state.filteredBlocks[i + 1].date || '';
            if (curr.localeCompare(next) < 0) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });

  it('sort by hash', () => {
    cy.get('.mina-table .head > span:nth-child(3)')
      .click()
      .window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].hash || '';
            const next = state.filteredBlocks[i + 1].hash || '';
            if (next.localeCompare(curr) < 0) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });

  it('sort by height', () => {
    cy.get('.mina-table .head > span:nth-child(4)')
      .click()
      .window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].height;
            const next = state.filteredBlocks[i + 1].height;
            if (next > curr) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });

  it('sort by sender', () => {
    cy.get('.mina-table .head > span:nth-child(5)')
      .click()
      .window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].sender || '';
            const next = state.filteredBlocks[i + 1].sender || '';
            if (next.localeCompare(curr) < 0) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });

  it('sort by receiver', () => {
    cy.get('.mina-table .head > span:nth-child(6)')
      .click()
      .window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].receiver || '';
            const next = state.filteredBlocks[i + 1].receiver || '';
            if (next.localeCompare(curr) < 0) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });

  it('sort by recv time', () => {
    cy.get('.mina-table .head > span:nth-child(7)')
      .click()
      .window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].receivedLatency === undefined ? state.filteredBlocks[i].receivedLatency : Number.MAX_VALUE;
            const next = state.filteredBlocks[i + 1].receivedLatency === undefined ? state.filteredBlocks[i + 1].receivedLatency : Number.MAX_VALUE;
            if (next > curr) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });

  it('sort by sent time', () => {
    cy.get('.mina-table .head > span:nth-child(8)')
      .click()
      .window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].sentLatency === undefined ? state.filteredBlocks[i].sentLatency : Number.MAX_VALUE;
            const next = state.filteredBlocks[i + 1].sentLatency === undefined ? state.filteredBlocks[i + 1].sentLatency : Number.MAX_VALUE;
            if (next > curr) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });

  it('sort by sent message kind', () => {
    cy.get('.mina-table .head > span:nth-child(9)')
      .click()
      .window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (state && state.blocks.length > 2) {
          let sorted = true;
          for (let i = 0; i < state.filteredBlocks.length - 1; i++) {
            const curr = state.filteredBlocks[i].messageKind || '';
            const next = state.filteredBlocks[i + 1].messageKind || '';
            if (next.localeCompare(curr) < 0) {
              sorted = false;
              break;
            }
          }
          expect(sorted).to.be.true;
        }
      });
  });
});
