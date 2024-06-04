import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { checkSorting, Sort, stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionWonSlotsState } from '@block-production/won-slots/block-production-won-slots.state';

const condition = (state: BlockProductionWonSlotsState): boolean => state && state.slots?.length > 0;
const getBPWonSlots = (store: Store<MinaState>): BlockProductionWonSlotsState => stateSliceAsPromise<BlockProductionWonSlotsState>(store, condition, 'blockProduction', 'wonSlots');
const execute = (callback: () => void) => {
  cy.wait('@statsRequest')
    .url()
    .then((url: string) => {
      if (url.includes('/block-production/won-slots')) {
        callback();
      }
    });
};

describe('BLOCK PRODUCTION WON SLOTS TABLE', () => {
  beforeEach(() => {
    cy
      .intercept('/stats/block_producer')
      .as('statsRequest')
      .visit(Cypress.config().baseUrl + '/block-production/won-slots');
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

  it('display slots in the table', () => {
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
  });

  it('by default, sort table by age', () => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'slotTime', Sort.ASC);
        }
      });
  });

  it('sort by name', () => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(1)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'message', Sort.ASC);
        }
      });
  });

  it('sort by height', () => {
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
  });

  it('sort by global slot', () => {
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
  });

  it('sort by transactions', () => {
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
  });

  it('sort by snark fees', () => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(6)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'snarkFees', Sort.ASC);
        }
      });
  });

  it('sort by snark coinbase rewards', () => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(7)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'coinbaseRewards', Sort.ASC);
        }
      });
  });

  it('sort by snark tx fees rewards', () => {
    cy.get('mina-block-production-won-slots-table .head > span:nth-child(8)')
      .click()
      .window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          checkSorting(state.filteredSlots, 'txFeesRewards', Sort.ASC);
        }
      });
  });
});

