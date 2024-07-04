import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NetworkConnectionsState } from '@network/connections/network-connections.state';
import { checkSorting, Sort, stateSliceAsPromise } from '../../../support/commands';
import { NetworkMessagesState } from '@network/messages/network-messages.state';

const condition = (state: NetworkConnectionsState) => state && state.connections?.length > 2;
const condition2 = (state: NetworkMessagesState) => state && state.messages?.length > 2;
const networkConnectionsState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkConnectionsState>(store, condition, 'network', 'connections');
const networkMessagesState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkMessagesState>(store, condition2, 'network', 'messages');


describe('NETWORK CONNECTIONS TABLE', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network/connections');
  });

  it('shows connections title', () => {
    cy.wait(2000)
      .window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (condition(state)) {
          cy.get('mina-toolbar span')
            .then((span: any) => expect(span).contain('Network'))
            .get('mina-toolbar .submenus a.active')
            .then((a: any) => expect(a.text().trim()).equals('connections'));
        }
      });
  });

  it('displays connections in the table', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (condition(state)) {
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
        if (condition(state)) {
          checkSorting(state.connections, 'timestamp', Sort.DSC);
        }
      });
  });

  it('click on remote address routes to network messages', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (condition(state)) {
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
        if (condition(state)) {
          cy.get('mina-network-connections .mina-table')
            .find('.row:not(.head) > span:nth-child(1)')
            .each((row: any, index: number) => {
              const id = state.connections[index].connectionId.toString();
              expect(row.text()).equal(id);
            });
        }
      });
  });

  it('clicking on address will redirect to messages page', () => {
    cy.window()
      .its('store')
      .then(networkConnectionsState)
      .then((state: NetworkConnectionsState) => {
        if (condition(state)) {
          cy.get('mina-network-connections .mina-table')
            .find('.row:not(.head) > span:nth-child(3)')
            .eq(1)
            .click()
            .wait(500)
            .url()
            .should('include', 'network/messages?addr=' + state.connections[1].addr)
            .wait(2000)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((messageState: NetworkMessagesState) => {
              if (condition2(messageState)) {
                expect(messageState.messages).length.to.be.greaterThan(0);
                expect(messageState.messages.every(m => m.address === state.connections[1].addr)).to.be.true;
              }
            });
        }
      });
  });

});
