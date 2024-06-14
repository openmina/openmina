import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { cyIsSubFeatureEnabled, stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';
import { BlockProductionEpochPaginationResponse } from '@block-production/overview/block-production-overview.service';
import { AppState } from '@app/app.state';

const condition = (state: BlockProductionOverviewState): boolean => state && state.epochs?.length > 0;
const getBPOverview = (store: Store<MinaState>): BlockProductionOverviewState => stateSliceAsPromise<BlockProductionOverviewState>(store, condition, 'blockProduction', 'overview');
const getAppState = (store: Store<MinaState>): AppState => stateSliceAsPromise<AppState>(store, () => true, 'app');
const getStore = () => cy.window().its('store');
const getConfig = () => cy.window().its('config');
const execute = (callback: () => void) => {
  getStore().then(getAppState).then((state: AppState) => {
    getConfig().then((config: any) => {
      if (cyIsSubFeatureEnabled(state.activeNode, 'block-production', 'overview', config.globalConfig)) {
        cy.wait('@allStatsRequest')
          .wait('@epochDetailsRequest')
          .wait('@slotsRequest')
          .url()
          .then((url: string) => {
            if (url.includes('/block-production/overview')) {
              callback();
            }
          });
      }
    });
  });
};
let epochDetails: BlockProductionEpochPaginationResponse;

describe('BLOCK PRODUCTION OVERVIEW SIDE PANEL', () => {
  beforeEach(() => {
    cy
      .visit(Cypress.config().baseUrl)
      .window()
      .its('store')
      .then(getAppState)
      .then((state: AppState) => {
        getConfig()
          .then((config: any) => {
            if (cyIsSubFeatureEnabled(state.activeNode, 'block-production', 'overview', config.globalConfig)) {
              cy
                .intercept(/\/epoch\/summary\/(?!.*\?limit=\d+)(latest|\d+)/, (req) => {
                  req.continue(res => {
                    epochDetails = res.body;
                  });
                })
                .as('epochDetailsRequest')
                .intercept('/summary')
                .as('allStatsRequest')
                .intercept(/\/epoch\/\d+/)
                .as('slotsRequest')
                .visit(Cypress.config().baseUrl + '/block-production/overview');
            }
          });
      });
  });

  it('show correct epoch number in tabs', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-side-panel .tab:first-child')
            .then(el => {
              expect(el.text().trim()).equals(`Epoch ${epochDetails.epoch_number}`);
            });
        }
      });
  }));

  it('show correct progress bar percentages', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const stats = {
            totalSlots: state.activeEpoch.details.totalSlots,
            canonical: state.activeEpoch.details.canonical,
            orphaned: state.activeEpoch.details.orphaned,
            missed: state.activeEpoch.details.missed,
            futureRights: state.activeEpoch.details.futureRights,
          };

          cy.get('mina-block-production-overview-side-panel .bar > div:nth-child(1)')
            .then(el => {
              const widthPercentage = ((stats.canonical / stats.totalSlots) * 100).toString().slice(0, 3);
              expect(el.attr('style')).to.satisfy((style: string) => {
                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
              });
            })
            .get('mina-block-production-overview-side-panel .bar > div:nth-child(2)')
            .then(el => {
              const widthPercentage = ((stats.orphaned / stats.totalSlots) * 100).toString().slice(0, 3);
              expect(el.attr('style')).to.satisfy((style: string) => {
                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
              });
            })
            .get('mina-block-production-overview-side-panel .bar > div:nth-child(3)')
            .then(el => {
              const widthPercentage = ((stats.missed / stats.totalSlots) * 100).toString().slice(0, 3);
              expect(el.attr('style')).to.satisfy((style: string) => {
                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
              });
            })
            .get('mina-block-production-overview-side-panel .bar > div:nth-child(4)')
            .then(el => {
              const widthPercentage = ((stats.futureRights / stats.totalSlots) * 100).toString().slice(0, 3);
              expect(el.attr('style')).to.satisfy((style: string) => {
                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
              });
            });
        }
      });
  }));

  it('show correct slots numbers', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const stats = {
            totalSlots: state.activeEpoch.details.totalSlots,
            canonical: state.activeEpoch.details.canonical,
            orphaned: state.activeEpoch.details.orphaned,
            missed: state.activeEpoch.details.missed,
            futureRights: state.activeEpoch.details.futureRights,
          };

          cy.get('mina-block-production-overview-side-panel .bar + div > span')
            .should('have.text', `${stats.canonical}`)
            .get('mina-block-production-overview-side-panel .bar + div + div > span')
            .should('have.text', `${stats.orphaned}`)
            .get('mina-block-production-overview-side-panel .bar + div + div + div > span')
            .should('have.text', `${stats.missed}`)
            .get('mina-block-production-overview-side-panel .bar + div + div + div + div > span')
            .should('have.text', `${stats.futureRights}`);
        }
      });
  }));

  it('show correct expected and earned rewards', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const stats = {
            earnedRewards: state.activeEpoch.details.earnedRewards,
            expectedRewards: state.activeEpoch.details.expectedRewards,
          };

          cy.get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(3) div')
            .should('have.text', `Expected ${stats.expectedRewards} Mina`)
            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(4) div')
            .should('have.text', `Earned ${stats.earnedRewards} Mina`);
        }
      });
  }));

  it('show correct produced, delegated and staked balances', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const extras = {
            balanceProducer: state.activeEpoch.details.balanceProducer,
            balanceDelegated: state.activeEpoch.details.balanceDelegated,
            balanceStaked: state.activeEpoch.details.balanceStaked,
          };

          cy.get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(6) div')
            .should('have.text', `Producer ${extras.balanceProducer} Mina`)
            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(7) div')
            .should('have.text', `Delegated ${extras.balanceDelegated} Mina`)
            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(8) div')
            .should('have.text', `Staked ${extras.balanceStaked} Mina`);
        }
      });
  }));

  it('show correct epoch stats', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const startSlot = state.activeEpoch.slots.find(slot => slot.globalSlot === state.activeEpoch.details.slotStart);
          const endSlot = state.activeEpoch.slots.find(slot => slot.globalSlot === state.activeEpoch.details.slotEnd);
          const extras = {
            epochStarted: startSlot ? toReadableDate(startSlot.time * 1000) : '-',
            epochEnds: endSlot ? toReadableDate(endSlot.time * 1000) : '-',
            slotsUsed: Math.round((state.activeEpoch.details.canonical + state.activeEpoch.details.orphaned + state.activeEpoch.details.missed) / state.activeEpoch.details.totalSlots * 100) + '%',
          };

          cy.get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(10) div')
            .should('have.text', `Epoch Started${extras.epochStarted}`)
            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(11) div')
            .should('have.text', `Epoch Finished${extras.epochEnds}`)
            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(12) div')
            .should('have.text', `% Slots Used${extras.slotsUsed}`);
        }
      });
  }));

  it('show correct progress bar percentages for all stats', () => execute(() => {
    cy.get('mina-block-production-overview-side-panel .tab:nth-child(2)')
      .click()
      .window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const stats = {
            totalSlots: state.allTimeStats.totalSlots,
            canonical: state.allTimeStats.canonical,
            orphaned: state.allTimeStats.orphaned,
            missed: state.allTimeStats.missed,
            futureRights: state.allTimeStats.futureRights,
          };

          cy.get('mina-block-production-overview-side-panel .bar > div:nth-child(1)')
            .then(el => {
              const widthPercentage = ((stats.canonical / stats.totalSlots) * 100).toString().slice(0, 3);
              expect(el.attr('style')).to.satisfy((style: string) => {
                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
              });
            })
            .get('mina-block-production-overview-side-panel .bar > div:nth-child(2)')
            .then(el => {
              const widthPercentage = ((stats.orphaned / stats.totalSlots) * 100).toString().slice(0, 3);
              expect(el.attr('style')).to.satisfy((style: string) => {
                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
              });
            })
            .get('mina-block-production-overview-side-panel .bar > div:nth-child(3)')
            .then(el => {
              const widthPercentage = ((stats.missed / stats.totalSlots) * 100).toString().slice(0, 3);
              expect(el.attr('style')).to.satisfy((style: string) => {
                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
              });
            })
            .get('mina-block-production-overview-side-panel .bar > div:nth-child(4)')
            .then(el => {
              const widthPercentage = ((stats.futureRights / stats.totalSlots) * 100).toString().slice(0, 3);
              expect(el.attr('style')).to.satisfy((style: string) => {
                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
              });
            });
        }
      });
  }));

  it('show correct slots numbers in all stats', () => execute(() => {
    cy.get('mina-block-production-overview-side-panel .tab:nth-child(2)')
      .click()
      .window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const stats = {
            totalSlots: state.allTimeStats.totalSlots,
            canonical: state.allTimeStats.canonical,
            orphaned: state.allTimeStats.orphaned,
            missed: state.allTimeStats.missed,
            futureRights: state.allTimeStats.futureRights,
          };

          cy.get('mina-block-production-overview-side-panel .bar + div > span')
            .should('have.text', `${stats.canonical}`)
            .get('mina-block-production-overview-side-panel .bar + div + div > span')
            .should('have.text', `${stats.orphaned}`)
            .get('mina-block-production-overview-side-panel .bar + div + div + div > span')
            .should('have.text', `${stats.missed}`)
            .get('mina-block-production-overview-side-panel .bar + div + div + div + div > span')
            .should('have.text', `${stats.futureRights}`);
        }
      });
  }));

  it('show correct expected and earned rewards for all stats', () => execute(() => {
    cy.get('mina-block-production-overview-side-panel .tab:nth-child(2)')
      .click()
      .window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const stats = {
            earnedRewards: state.allTimeStats.earnedRewards,
            expectedRewards: state.allTimeStats.expectedRewards,
          };

          cy.get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(3) div')
            .should('have.text', `Expected ${stats.expectedRewards} Mina`)
            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(4) div')
            .should('have.text', `Earned ${stats.earnedRewards} Mina`);
        }
      });
  }));
});

function toReadableDate(val: number): string {
  const date = new Date(val);
  const options: any = {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    day: '2-digit',
    month: 'short',
    year: '2-digit',
  };
  const timeString = date.toLocaleTimeString('en-GB', options);
  const dateString = date.toLocaleDateString('en-GB', options);
  return `${timeString.split(', ')[1]}, ${dateString.split(',')[0]}`;
}
