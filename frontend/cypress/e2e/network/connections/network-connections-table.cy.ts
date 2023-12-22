import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NetworkConnectionsState } from '@network/connections/network-connections.state';
import { checkSorting, Sort, stateSliceAsPromise } from '../../../support/commands';

const condition = (state: NetworkConnectionsState) => state && state.connections.length > 2;
const networkConnectionsState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkConnectionsState>(store, condition, 'network', 'connections');


describe('NETWORK CONNECTIONS TABLE', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network/connections');
  });

  it('shows correct active page', () => {
    cy.wait(1000)
      .get('mina-toolbar .toolbar > div:first-child > span')
      .then((span: any) => expect(span.text()).equal('Network'))
      .get('mina-submenu-tabs a.active')
      .then((a: any) => expect(a.text().trim().toLowerCase()).equals('connections'));
  });

  it('displays connections in the table', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (state && state.connections.length > 2) {
          expect(state.connections.length).above(2);
          cy.get('mina-network-connections .mina-table')
            .find('.row:not(.head)')
            .should('have.length.above', 2);
        }
      });
  });

  it('assert that connections are sorted by date', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (state && state.connections.length > 2) {
          checkSorting(state.connections, 'timestamp', Sort.ASC);
        }
      });
  });

  it('click on remote address routes to network messages', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (state && state.connections.length > 2) {
          cy.get('mina-network-connections .mina-table')
            .find('.row:not(.head)')
            .first()
            .find('span:nth-child(3) span')
            .click()
            .wait(1000)
            .url()
            .should('include', '/network/messages?addr=' + state.connections[0].addr);
        }
      });
  });

  it('for each row assert that the id is displayed correctly', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (state && state.connections.length > 2) {
          cy.get('mina-network-connections .mina-table')
            .find('.row:not(.head) > span:nth-child(1)')
            .each((row: any, index: number) => {
              const id = state.connections[index].connectionId.toString();
              expect(row.text()).equal(id);
            });
        }
      });
  });

});
