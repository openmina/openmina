import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { cyIsSubFeatureEnabled, stateSliceAsPromise } from '../../../support/commands';
import { AppState } from '@app/app.state';
import { BlockProductionWonSlotsState } from '@block-production/won-slots/block-production-won-slots.state';
import {
  BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';

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

describe('BLOCK PRODUCTION WON SLOTS SIDE PANEL', () => {
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

  it('show side panel by default as open', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-side-panel div')
            .should('be.visible');
        }
      });
  }));

  it('have slot preselected if a slot is commited or scheduled', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          if (state.slots.some(s => s.status === BlockProductionWonSlotsStatus.Committed || s.status === BlockProductionWonSlotsStatus.Scheduled)) {
            expect(state.activeSlot).to.not.be.null;
            expect(state.activeSlot).to.not.be.undefined;
          }
          cy.get('mina-block-production-won-slots-side-panel > .h-minus-xl > div:first-child > div.h-lg:first-child')
            .should('have.text', 'Global slot' + state.activeSlot.globalSlot);
        }
      });
  }));

  it('selecting first slot should display its data in the side panel', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPWonSlots)
      .then((state: BlockProductionWonSlotsState) => {
        if (condition(state)) {
          cy.get('mina-block-production-won-slots-table .row:not(.head)')
            .eq(1)
            .click()
            .then(row => {
              const globalSlot = row.find('> span').eq(3).text();
              const expectedActiveSlot = state.slots.find(s => s.globalSlot.toString() === globalSlot);
              expect(expectedActiveSlot.globalSlot.toString()).to.equal(globalSlot);
              cy.get('mina-block-production-won-slots-side-panel > .h-minus-xl > div:first-child > div.h-lg:first-child')
                .should('have.text', 'Global slot' + expectedActiveSlot.globalSlot)
                .get('mina-block-production-won-slots-side-panel > div:first-child > span')
                .then(span => expect(row.find('> span').eq(0).text()).to.contain(span.text()))
                .get('mina-block-production-won-slots-side-panel > div:first-child > span')
                .should('have.text', expectedActiveSlot.message)
                .window()
                .its('store')
                .then(getBPWonSlots)
                .then((state: BlockProductionWonSlotsState) => {
                  expect(state.activeSlot.globalSlot).to.equal(expectedActiveSlot.globalSlot);
                  expect(state.activeSlot.height).to.equal(expectedActiveSlot.height);
                  console.log(expectedActiveSlot.times);
                })
                .get('mina-block-production-won-slots-side-panel .percentage')
                .should('have.text', ([
                  expectedActiveSlot.times?.stagedLedgerDiffCreate,
                  expectedActiveSlot.times?.produced,
                  expectedActiveSlot.times?.proofCreate,
                  expectedActiveSlot.times?.blockApply,
                  expectedActiveSlot.times?.committed,
                ].filter(t => t !== null && t !== undefined).length * 20) + '%');

            });
        }
      });
  }));

});
