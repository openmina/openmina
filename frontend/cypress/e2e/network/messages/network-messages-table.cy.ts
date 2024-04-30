import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NetworkMessage } from '@shared/types/network/messages/network-message.type';
import { stateSliceAsPromise } from '../../../support/commands';
import { NetworkMessagesState } from '@network/messages/network-messages.state';

const condition = (state: NetworkMessagesState) => state && state.messages?.length > 20;
const networkMessagesState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkMessagesState>(store, condition, 'network', 'messages');


describe('NETWORK MESSAGES TABLE', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network');
  });

  it('displays network title', () => {
    cy.wait(1000)
      .window()
      .its('store')
      .then(networkMessagesState)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          cy.get('mina-toolbar span')
            .then((span: any) => expect(span).contain('Network'));
        }
      });
  });

  it('displays messages in the table', () => {
    cy.wait(1000)
      .window()
      .its('store')
      .then(networkMessagesState)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          expect(state.messages.length).above(20);
          cy.get('.mina-table')
            .get('.row')
            .should('have.length.above', 15);
        }
      });
  });

  it('toggle address filter on address click', () => {
    cy.window()
      .its('store')
      .then(networkMessagesState)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          cy.wait(1000)
            .get('.mina-table .row:last-child span:nth-child(3)')
            .click()
            .get('.filter-row div:nth-child(2) button').should('have.length', 1)
            .url().should('contain', 'addr=' + state.messages[state.messages.length - 1].address)
            .wait(2000)
            .get('.mina-table .row:last-child span:nth-child(3)')
            .click()
            .get('.filter-row div:nth-child(2) button').should('have.length', 0)
            .url().should('not.contain', 'addr=');
        }
      });
  });

  it('select message on click', () => {
    let clickedMessage: NetworkMessage;
    cy.window()
      .its('store')
      .then(networkMessagesState)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          clickedMessage = state.messages[state.messages.length - 2];
          cy.wait(1000)
            .get('.mina-table .row')
            .eq(-2)
            .find('span:nth-child(2)')
            .click()
            .wait(1000)
            .url().should('contain', '/' + clickedMessage.id)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((network: NetworkMessagesState) => {
              expect(network.activeRow).equals(clickedMessage);
            });
        }
      });
  });

  it('deselect message on closing side panel', () => {
    let clickedMessage: NetworkMessage;
    cy.window()
      .its('store')
      .then(networkMessagesState)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          clickedMessage = state.messages[state.messages.length - 2];
          cy.wait(1000)
            .get('.mina-table .row')
            .eq(-2)
            .find('span:nth-child(2)')
            .click()
            .wait(1000)
            .url().should('contain', '/' + clickedMessage.id)
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((network: NetworkMessagesState) => {
              expect(network.activeRow).equals(clickedMessage);
            })
            .get('mina-network-messages-side-panel > div:first-child .mina-icon')
            .click()
            .wait(500)
            .url().should('not.contain', 'network/messages/')
            .get('mina-network-messages-side-panel button')
            .should('not.be.visible')
            .window()
            .its('store')
            .then(networkMessagesState)
            .then((network: NetworkMessagesState) => {
              expect(network.activeRow).to.be.undefined;
            });
        }
      });
  });

  // it.only('stop getting messages if there is an ongoing messages request', () => {
  //   let responseCount: number = 0;
  //   cy.get('.pause-button')
  //     .click()
  //     .wait(8000)
  //     .intercept('/messages?limit=1000&direction=reverse', req => {
  //       req.continue(res => {
  //         res.delay = 30000;
  //         responseCount++;
  //         res.send();
  //       });
  //     })
  //     .as('getMessages')
  //     .get('.live-button')
  //     .click()
  //     .wait(30000)
  //     .then(() => {
  //       expect(responseCount).equals(1);
  //     });
  // });
});
