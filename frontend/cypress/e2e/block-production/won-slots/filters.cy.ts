import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { cyIsSubFeatureEnabled, stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionWonSlotsState } from '@block-production/won-slots/block-production-won-slots.state';
import {
  BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
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

describe('BLOCK PRODUCTION WON SLOTS FILTERS', () => {
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
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div:nth-child(2)')
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
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div:nth-child(3)')
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
          cy.get('mina-block-production-won-slots-filters .overflow-hidden > div:nth-child(4)')
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
          if (hasDropped(state)) {
            cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.aware-primary', { timeout: 500 })
              .click();
          }
          if (hasUpcoming(state)) {
            cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.bg-container.primary', { timeout: 500 })
              .click();
          }
          cy
            .wait(1000)
            .window()
            .its('store')
            .then(getBPWonSlots)
            .then((state: BlockProductionWonSlotsState) => {
              const producing = state.slots.filter(s => s.active || s.status === BlockProductionWonSlotsStatus.Committed).length;
              const scheduled = state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Scheduled).length;
              expect(state.filteredSlots.length).equals(state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Canonical).length + producing + scheduled);
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
          if (hasCanonical(state)) {
            cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.success-primary')
              .click();
          }
          if (hasUpcoming(state)) {
            cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.bg-container.primary')
              .click();
          }
          cy
            .wait(1000)
            .window()
            .its('store')
            .then(getBPWonSlots)
            .then((state: BlockProductionWonSlotsState) => {
              const producing = state.slots.filter(s => s.active || s.status === BlockProductionWonSlotsStatus.Committed).length;
              const scheduled = state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Scheduled).length;
              expect(state.filteredSlots.length).equals(state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Orphaned || s.status === BlockProductionWonSlotsStatus.Discarded).length + producing + scheduled);
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
          if (hasCanonical(state)) {
            cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.success-primary')
              .click();
          }
          if (hasDropped(state)) {
            cy.get('mina-block-production-won-slots-filters .overflow-hidden > div.aware-primary')
              .click();
          }
          cy
            .wait(1000)
            .window()
            .its('store')
            .then(getBPWonSlots)
            .then((state: BlockProductionWonSlotsState) => {
              const producing = state.slots.filter(s => s.active || s.status === BlockProductionWonSlotsStatus.Committed).length;
              const scheduled = state.slots.filter(s => s.status === BlockProductionWonSlotsStatus.Scheduled).length;
              expect(state.filteredSlots.length).equals(state.slots.filter(s => !s.status).length + producing + scheduled);
            });
        }
      });
  }));
});


function hasCanonical(state: BlockProductionWonSlotsState): boolean {
  return state.slots.some(s => s.status === BlockProductionWonSlotsStatus.Canonical);
}

function hasDropped(state: BlockProductionWonSlotsState): boolean {
  return state.slots.some(s => s.status === BlockProductionWonSlotsStatus.Discarded || s.status === BlockProductionWonSlotsStatus.Orphaned);
}

function hasUpcoming(state: BlockProductionWonSlotsState): boolean {
  return state.slots.some(s => !s.status);
}
