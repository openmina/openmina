import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionWonSlotsState } from '@block-production/won-slots/block-production-won-slots.state';
import {
  BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';

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

describe('BLOCK PRODUCTION WON SLOTS FILTERS', () => {
  beforeEach(() => {
    cy
      .intercept('/stats/block_producer')
      .as('statsRequest')
      .visit(Cypress.config().baseUrl + '/block-production/won-slots');
  });

  it('show correct number of won slots', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div:first-child')
            .then((div: any) => expect(div.text()).equals(`${state.slots.length} Won slots`));
        }
      });
  }));

  it('show correct number of produced blocks', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.success-primary')
            .then((div: any) => expect(div.text().trim()).equals(
              `${state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Canonical).length} Produced`,
            ));
        }
      });
  }));

  it('show correct number of dropped blocks', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.aware-primary')
            .then((div: any) => expect(div.text().trim()).equals(
              `${state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Discarded || s.status === BlockProductionWonSlotsStatus.Orphaned).length} Dropped`,
            ));
        }
      });
  }));

  it('show correct number of upcoming block', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.bg-container')
            .then((div: any) => expect(div.text().trim()).equals(
              `${state.slots.filter(s => !s.status || s.status === BlockProductionWonSlotsStatus.Scheduled).length} Upcoming`,
            ));
        }
      });
  }));

  it('show only produced blocks', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.aware-primary')
            .click()
            .get('mina-block-production-won-slots-filters .overflow-hidden > div.bg-container.primary')
            .click()
            .window()
            .its('store')
            .then(getBPWonSlots)
            .then((state: BlockProductionWonSlotsState) => {
              const producing = state.slots.filter(s => s.active || s.status === BlockProductionWonSlotsStatus.Committed).length;
              expect(state.filteredSlots.length).equals(state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Canonical).length + producing);
            });
        }
      });
  }));

  it('show only dropped blocks', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.success-primary')
            .click()
            .get('mina-block-production-won-slots-filters .overflow-hidden > div.bg-container.primary')
            .click()
            .window()
            .its('store')
            .then(getBPWonSlots)
            .then((state: BlockProductionWonSlotsState) => {
              const producing = state.slots.filter(s => s.active || s.status === BlockProductionWonSlotsStatus.Committed).length;
              expect(state.filteredSlots.length).equals(state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Orphaned || s.status === BlockProductionWonSlotsStatus.Discarded).length + producing);
            });
        }
      });
  }));

  it('show only upcoming blocks', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.success-primary')
            .click()
            .get('mina-block-production-won-slots-filters .overflow-hidden > div.aware-primary')
            .click()
            .window()
            .its('store')
            .then(getBPWonSlots)
            .then((state: BlockProductionWonSlotsState) => {
              const producing = state.slots.filter(s => s.active || s.status === BlockProductionWonSlotsStatus.Committed).length;
              expect(state.filteredSlots.length).equals(state.slots.filter(s => !s.status || s.status === BlockProductionWonSlotsStatus.Scheduled).length + producing);
            });
        }
      });
  }));
});

