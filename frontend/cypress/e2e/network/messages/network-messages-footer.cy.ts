import { NetworkMessagesState } from '@network/messages/network-messages.state';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';

const condition = (state: NetworkMessagesState) => state && state.messages?.length > 20;
const getNetworkMessages = (store: Store<MinaState>) => stateSliceAsPromise<NetworkMessagesState>(store, condition, 'network', 'messages');

describe('NETWORK MESSAGES FOOTER', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network');
  });

  it('in live mode messages are retrieved every 10 seconds', () => {
    let responseCount: number = 0;
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          expect(state.stream).to.be.true;
          cy
            .intercept('/messages?limit=1000&direction=reverse', req => {
              req.continue(res => {
                responseCount++;
                res.send();
              });
            })
            .as('getMessages')
            .wait(25000)
            .then(() => {
              expect(responseCount).equals(2);
            });
        }
      });
  });

  it('in pause mode no messages are retrieved', () => {
    let responseCount: number = 0;
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          cy.get('mina-network-messages-table-footer .pause-button')
            .click()
            .intercept('/messages?limit=1000&direction=reverse', req => {
              req.continue(res => {
                responseCount++;
                res.send();
              });
            })
            .as('getMessages')
            .wait(15000)
            .then(() => expect(responseCount).equals(0));
        }
      });
  });

  it('scroll table to top', () => {
    let firstItemID: string;
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          cy.get('mina-network-messages-table-footer button.icon-button:nth-child(4)')
            .click()
            .wait(1000)
            .get('mina-network-messages-table cdk-virtual-scroll-viewport .row:nth-child(1) > span:nth-child(1)')
            .then((span: JQuery<HTMLSpanElement>) => firstItemID = span.text())
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                expect(state.messages[0].id.toString()).to.equal(firstItemID);
                expect(state.stream).to.be.false;
              }
            });
        }
      });
  });

  it('jump to first page', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          cy.intercept('/messages?limit=1000&direction=forward')
            .as('getMessages')
            .get('mina-network-messages-table-footer button.icon-button:nth-child(5)')
            .click()
            .wait(1000)
            .wait('@getMessages', { timeout: 10000 })
            .get('mina-network-messages-table-footer button.icon-button:nth-child(5)')
            .should('be.disabled')
            .get('mina-network-messages-table-footer button.icon-button:nth-child(6)')
            .should('be.disabled')
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                expect(state.messages[0].id).to.equal(0);
                if (state.messages.length === state.limit) {
                  cy.get('mina-network-messages-table-footer button.icon-button:nth-child(7)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(8)')
                    .should('not.be.disabled');
                }
              }
            });
        }
      });
  });

  it('jump to previous page', () => {
    let firstMessageID: number;
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          cy.intercept('/messages?limit=1000&direction=reverse*')
            .as('getMessages')
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((network: NetworkMessagesState) => firstMessageID = network.messages[0].id)
            .get('mina-network-messages-table-footer button.icon-button:nth-child(6)')
            .click()
            .wait('@getMessages', { timeout: 10000 })
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                expect(state.messages[0].id).to.equal(firstMessageID - state.limit);
                if (state.messages.length === state.limit && state.messages[0].id > 1) {
                  cy.get('mina-network-messages-table-footer button.icon-button:nth-child(5)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(6)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(7)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(8)')
                    .should('not.be.disabled');
                }
              }
            });
        }
      });
  });

  it('jump to next page', () => {
    let firstMessageID: number;
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          cy.intercept('/messages?limit=1000&direction=forward*')
            .as('getMessages')
            .get('mina-network-messages-table-footer button.icon-button:nth-child(5)')
            .click()
            .wait('@getMessages', { timeout: 10000 })
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((network: NetworkMessagesState) => firstMessageID = network.messages[0].id)
            .get('mina-network-messages-table-footer button.icon-button:nth-child(7)')
            .click()
            .wait('@getMessages', { timeout: 10000 })
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                expect(state.messages[0].id).to.equal(firstMessageID + state.limit);
                if (state.messages.length === state.limit) {
                  cy.get('mina-network-messages-table-footer button.icon-button:nth-child(5)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(6)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(7)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(8)')
                    .should('not.be.disabled');
                }
              }
            });
        }
      });
  });

  it('jump to last page', () => {
    let firstMessageID: number;
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          cy.intercept('/messages?limit=1000&direction=forward*')
            .as('getFirstPageMessages')
            .intercept('/messages?limit=1000&direction=reverse*')
            .as('getLastPageMessages')
            .get('mina-network-messages-table-footer button.icon-button:nth-child(5)')
            .click()
            .wait('@getFirstPageMessages', { timeout: 10000 })
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((network: NetworkMessagesState) => firstMessageID = network.messages[network.messages.length - 1].id)
            .get('mina-network-messages-table-footer button.icon-button:nth-child(8)')
            .click()
            .wait('@getLastPageMessages', { timeout: 10000 })
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                expect(state.messages[0].id).to.be.greaterThan(firstMessageID);
                if (state.messages.length === state.limit) {
                  cy.get('mina-network-messages-table-footer button.icon-button:nth-child(5)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(6)')
                    .should('not.be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(7)')
                    .should('be.disabled')
                    .get('mina-network-messages-table-footer button.icon-button:nth-child(8)')
                    .should('be.disabled');
                }
              }
            });
        }
      });
  });

  it('render present day values in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const yearToday = new Date().getFullYear();
          const monthToday = new Date().getMonth() + 1;
          const dayToday = new Date().getDate();
          cy.get('.cdk-overlay-container mina-interval-select')
            .should('not.exist')
            .get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select')
            .should('exist')
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearToday)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthToday)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayToday)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearToday)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthToday)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayToday);
        }
      });
  });

  it('set the current time in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form button')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form button')
            .click()
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 1m interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateOneMinuteAgo = new Date(Date.now() - 60000);
          const yearOneMinuteAgo = dateOneMinuteAgo.getFullYear();
          const monthOneMinuteAgo = dateOneMinuteAgo.getMonth() + 1;
          const dayOneMinuteAgo = dateOneMinuteAgo.getDate();
          const hoursOneMinuteAgo = dateOneMinuteAgo.getHours();
          const minutesOneMinuteAgo = dateOneMinuteAgo.getMinutes();
          let secondsOneMinuteAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(2)')
            .click()
            .then(() => secondsOneMinuteAgo = new Date(Date.now() - 60000).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearOneMinuteAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthOneMinuteAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayOneMinuteAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursOneMinuteAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesOneMinuteAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsOneMinuteAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 5m interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateFiveMinutesAgo = new Date(Date.now() - (5 * 60000));
          const yearFiveMinutesAgo = dateFiveMinutesAgo.getFullYear();
          const monthFiveMinutesAgo = dateFiveMinutesAgo.getMonth() + 1;
          const dayFiveMinutesAgo = dateFiveMinutesAgo.getDate();
          const hoursFiveMinutesAgo = dateFiveMinutesAgo.getHours();
          const minutesFiveMinutesAgo = dateFiveMinutesAgo.getMinutes();
          let secondsFiveMinutesAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(3)')
            .click()
            .then(() => secondsFiveMinutesAgo = new Date(Date.now() - (5 * 60000)).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearFiveMinutesAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthFiveMinutesAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayFiveMinutesAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursFiveMinutesAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesFiveMinutesAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsFiveMinutesAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 30m interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateThirtyMinutesAgo = new Date(Date.now() - (30 * 60000));
          const yearThirtyMinutesAgo = dateThirtyMinutesAgo.getFullYear();
          const monthThirtyMinutesAgo = dateThirtyMinutesAgo.getMonth() + 1;
          const dayThirtyMinutesAgo = dateThirtyMinutesAgo.getDate();
          const hoursThirtyMinutesAgo = dateThirtyMinutesAgo.getHours();
          const minutesThirtyMinutesAgo = dateThirtyMinutesAgo.getMinutes();
          let secondsThirtyMinutesAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(4)')
            .click()
            .then(() => secondsThirtyMinutesAgo = new Date(Date.now() - (30 * 60000)).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearThirtyMinutesAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthThirtyMinutesAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayThirtyMinutesAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursThirtyMinutesAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesThirtyMinutesAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsThirtyMinutesAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 1h interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateOneHourAgo = new Date(Date.now() - (60 * 60000));
          const yearOneHourAgo = dateOneHourAgo.getFullYear();
          const monthOneHourAgo = dateOneHourAgo.getMonth() + 1;
          const dayOneHourAgo = dateOneHourAgo.getDate();
          const hoursOneHourAgo = dateOneHourAgo.getHours();
          const minutesOneHourAgo = dateOneHourAgo.getMinutes();
          let secondsOneHourAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(5)')
            .click()
            .then(() => secondsOneHourAgo = new Date(Date.now() - (60 * 60000)).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearOneHourAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthOneHourAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayOneHourAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursOneHourAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesOneHourAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsOneHourAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 12h interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateTwelveHoursAgo = new Date(Date.now() - (720 * 60000));
          const yearTwelveHoursAgo = dateTwelveHoursAgo.getFullYear();
          const monthTwelveHoursAgo = dateTwelveHoursAgo.getMonth() + 1;
          const dayTwelveHoursAgo = dateTwelveHoursAgo.getDate();
          const hoursTwelveHoursAgo = dateTwelveHoursAgo.getHours();
          const minutesTwelveHoursAgo = dateTwelveHoursAgo.getMinutes();
          let secondsTwelveHoursAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(6)')
            .click()
            .then(() => secondsTwelveHoursAgo = new Date(Date.now() - (720 * 60000)).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearTwelveHoursAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthTwelveHoursAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayTwelveHoursAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursTwelveHoursAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesTwelveHoursAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsTwelveHoursAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 1d interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateOneDayAgo = new Date(Date.now() - (1440 * 60000));
          const yearOneDayAgo = dateOneDayAgo.getFullYear();
          const monthOneDayAgo = dateOneDayAgo.getMonth() + 1;
          const dayOneDayAgo = dateOneDayAgo.getDate();
          const hoursOneDayAgo = dateOneDayAgo.getHours();
          const minutesOneDayAgo = dateOneDayAgo.getMinutes();
          let secondsOneDayAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(7)')
            .click()
            .then(() => secondsOneDayAgo = new Date(Date.now() - (1440 * 60000)).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearOneDayAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthOneDayAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayOneDayAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursOneDayAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesOneDayAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsOneDayAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 2d interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateTwoDaysAgo = new Date(Date.now() - (2880 * 60000));
          const yearTwoDaysAgo = dateTwoDaysAgo.getFullYear();
          const monthTwoDaysAgo = dateTwoDaysAgo.getMonth() + 1;
          const dayTwoDaysAgo = dateTwoDaysAgo.getDate();
          const hoursTwoDaysAgo = dateTwoDaysAgo.getHours();
          const minutesTwoDaysAgo = dateTwoDaysAgo.getMinutes();
          let secondsTwoDaysAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(8)')
            .click()
            .then(() => secondsTwoDaysAgo = new Date(Date.now() - (2880 * 60000)).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearTwoDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthTwoDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayTwoDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursTwoDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesTwoDaysAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsTwoDaysAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 7d interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateSevenDaysAgo = new Date(Date.now() - (10080 * 60000));
          const yearSevenDaysAgo = dateSevenDaysAgo.getFullYear();
          const monthSevenDaysAgo = dateSevenDaysAgo.getMonth() + 1;
          const daySevenDaysAgo = dateSevenDaysAgo.getDate();
          const hoursSevenDaysAgo = dateSevenDaysAgo.getHours();
          const minutesSevenDaysAgo = dateSevenDaysAgo.getMinutes();
          let secondsSevenDaysAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(9)')
            .click()
            .then(() => secondsSevenDaysAgo = new Date(Date.now() - (10080 * 60000)).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearSevenDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthSevenDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', daySevenDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursSevenDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesSevenDaysAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsSevenDaysAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('set 30d interval in interval picker component', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          const date = new Date();
          const yearNow = date.getFullYear();
          const monthNow = date.getMonth() + 1;
          const dayNow = date.getDate();
          const hourNow = date.getHours();
          const minuteNow = date.getMinutes();
          let secondNow: number;
          const dateThirtyDaysAgo = new Date(Date.now() - (43200 * 60000));
          const yearThirtyDaysAgo = dateThirtyDaysAgo.getFullYear();
          const monthThirtyDaysAgo = dateThirtyDaysAgo.getMonth() + 1;
          const dayThirtyDaysAgo = dateThirtyDaysAgo.getDate();
          const hoursThirtyDaysAgo = dateThirtyDaysAgo.getHours();
          const minutesThirtyDaysAgo = dateThirtyDaysAgo.getMinutes();
          let secondsThirtyDaysAgo: number;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(10)')
            .click()
            .then(() => secondsThirtyDaysAgo = new Date(Date.now() - (43200 * 60000)).getSeconds())
            .then(() => secondNow = new Date().getSeconds())
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(5)')
            .should('have.value', yearThirtyDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(6)')
            .should('have.value', monthThirtyDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(7)')
            .should('have.value', dayThirtyDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(2)')
            .should('have.value', hoursThirtyDaysAgo)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minutesThirtyDaysAgo, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(2) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondsThirtyDaysAgo, 1);
            })
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(5)')
            .should('have.value', yearNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(6)')
            .should('have.value', monthNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(7)')
            .should('have.value', dayNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(2)')
            .should('have.value', hourNow)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(3)')
            .invoke('val')
            .then((value: string | number | string[]) => expect(Number(value)).to.be.approximately(minuteNow, 1))
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(3) form input:nth-child(4)')
            .invoke('val')
            .then((value: string | number | string[]) => {
              expect(Number(value)).to.be.approximately(secondNow, 1);
            });
        }
      });
  });

  it('get messages in last 1 minute', () => {
    cy.window()
      .its('store')
      .then(getNetworkMessages)
      .then((state: NetworkMessagesState) => {
        if (condition(state)) {
          let date: Date;
          let dateOneMinuteAgo: Date;
          cy.get('mina-network-messages-table-footer button:last-child')
            .click()
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(1) button:nth-child(2)')
            .click()
            .wait(40)
            .then(() => {
              date = new Date();
              dateOneMinuteAgo = new Date(Date.now() - 60000);
            })
            .wait(400)
            .get('.cdk-overlay-container mina-interval-select .container div:nth-child(4) button')
            .click()
            .get('.cdk-overlay-container mina-interval-select')
            .should('not.exist')
            .get('mina-network-messages-table-footer button:last-child')
            .then((btn: any) => {
              const twoDigit = (val: number) => val < 10 ? `0${val}` : val;

              const from = dateOneMinuteAgo.toLocaleDateString('en-us', { month: 'short', day: 'numeric' })
                + ', '
                + dateOneMinuteAgo.getHours() + ':' + twoDigit(dateOneMinuteAgo.getMinutes()) + ':';
              let to = date.toLocaleDateString('en-us', { month: 'short', day: 'numeric' })
                + ', '
                + date.getHours() + ':' + twoDigit(date.getMinutes()) + ':' + twoDigit(date.getSeconds());
              if (from.split(',')[0] === to.split(',')[0]) {
                to = date.getHours() + ':' + twoDigit(date.getMinutes()) + ':';
              }
              expect(btn.text().substring(0, (btn.text().length - ' close'.length))).to.include(from);
              expect(btn.text().substring(0, (btn.text().length - ' close'.length))).to.include(to);
              cy.url().should('include', '?from=' + Math.floor(dateOneMinuteAgo.getTime() / 10000));
              cy.url().should('include', '&to=' + Math.floor(date.getTime() / 10000));
            })
            .wait(500)
            .window()
            .its('store')
            .then(getNetworkMessages)
            .then((state: NetworkMessagesState) => {
              if (condition(state)) {
                expect(state.stream).to.be.false;
                expect(state.messages).to.not.be.empty;
              }
            });
        }
      });
  });
});
