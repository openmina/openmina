import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NetworkConnectionsState } from '@network/connections/network-connections.state';
import { stateSliceAsPromise } from '../../../support/commands';

const condition = (state: NetworkConnectionsState) => state && state.connections.length > 2;
const networkConnectionsState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkConnectionsState>(store, condition, 'network', 'connections');


describe('NETWORK CONNECTIONS SIDE PANEL', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network/connections');
  });

  it('click on row should open side panel', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (state && state.connections.length > 2) {
          cy
            .get('mina-network-connections-side-panel mina-json-viewer')
            .should('not.be.visible')
            .get('mina-network-connections .mina-table')
            .find('.row:not(.head)')
            .first()
            .click()
            .wait(1000)
            .url()
            .should('include', '/network/connections/0')
            .get('mina-network-connections-side-panel mina-json-viewer')
            .should('be.visible')
            .get('mina-network-connections .mina-table')
            .find('.row:not(.head)')
            .first()
            .should('have.class', 'active');
        }
      });
  });

  it('close side panel', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (state && state.connections.length > 2) {
          cy
            .get('mina-network-connections-side-panel mina-json-viewer')
            .should('not.be.visible')
            .get('mina-network-connections .mina-table')
            .find('.row:not(.head)')
            .first()
            .click()
            .wait(1000)
            .get('mina-network-connections-side-panel mina-json-viewer')
            .should('be.visible')
            .get('mina-network-connections .mina-table')
            .find('.row:not(.head)')
            .first()
            .should('have.class', 'active')
            .get('mina-network-connections-side-panel > div > span.mina-icon')
            .click()
            .wait(1000)
            .get('mina-network-connections-side-panel mina-json-viewer')
            .should('not.be.visible')
            .get('mina-network-connections .mina-table')
            .find('.row:not(.head)')
            .first()
            .should('not.have.class', 'active');
        }
      });
  });

});
