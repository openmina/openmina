import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { checkSorting, cyIsSubFeatureEnabled, Sort, stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionWonSlotsState } from '@block-production/won-slots/block-production-won-slots.state';
import { AppState } from '@app/app.state';

const condition = (state: BlockProductionWonSlotsState): boolean => state && state.slots?.length > 0;
const getBPWonSlots = (store: Store<MinaState>): BlockProductionWonSlotsState => stateSliceAsPromise<BlockProductionWonSlotsState>(store, condition, 'blockProduction', 'wonSlots');
const getAppState = (store: Store<MinaState>): AppState => stateSliceAsPromise<AppState>(store, () => true, 'app');
const getStore = () => cy.window().its('store');
const getConfig = () => cy.window().its('config');
const execute = (callback: () => void) => {
  getStore().then(getAppState).then((state: AppState) => {
    getConfig().then((config: any) => {
      if (cyIsSubFeatureEnabled(state.activeNode, 'block-production', 'won-slots', config.globalConfig)) {
        cy.wait('@statsRequest')
          .url()
          .then((url: string) => {
            if (url.includes('/block-production/won-slots')) {
              callback();
            }
          });
      }
    });
  });
};

describe('BLOCK PRODUCTION WON SLOTS TABLE', () => {
  beforeEach(() => {
    cy
      .visit(Cypress.config().baseUrl)
      .window()
      .its('store')
      .then(getAppState)
      .then((state: AppState) => {
        getConfig()
          .then((config: any) => {
            if (cyIsSubFeatureEnabled(state.activeNode, 'block-production', 'won-slots', config.globalConfig)) {
              cy
                .intercept('/stats/block_producer')
                .as('statsRequest')
                .visit(Cypress.config().baseUrl + '/block-production/won-slots');
            }
          });
      });
  });

  it('have correct title', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-toolbar span')
            .then((span: any) => expect(span).contain('Block Production'));
        }
      });
  }));

  it('display slots in the table', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          expect(state.slots.length).above(0);
          cy.get('mina-block-production-won-slots .mina-table')
            .get('.row')
            .should('have.length.above', 0);
        }
      });
  }));

  it('by default, sort table by age', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'slotTime', Sort.ASC);
        }
      });
  }));

  it('sort by name', () => execute(() => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(1)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'message', Sort.DSC);
        }
      });
  }));

  it('sort by height', () => execute(() => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(3)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'height', Sort.ASC);
        }
      });
  }));

  it('sort by global slot', () => execute(() => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(4)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'globalSlot', Sort.ASC);
        }
      });
  }));

  it('sort by transactions', () => execute(() => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(5)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'transactionsTotal', Sort.ASC);
        }
      });
  }));

  it('sort by snark fees', () => execute(() => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(7)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'snarkFees', Sort.ASC);
        }
      });
  }));

  it('sort by snark coinbase rewards', () => execute(() => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(8)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'coinbaseRewards', Sort.ASC);
        }
      });
  }));

  it('sort by snark tx fees rewards', () => execute(() => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(9)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'txFeesRewards', Sort.ASC);
        }
      });
  }));
});

