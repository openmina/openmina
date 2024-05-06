import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';
import { BlockProductionEpochPaginationResponse } from '@block-production/overview/block-production-overview.service';

const condition = (state: BlockProductionOverviewState): boolean => state && state.epochs?.length > 0;
const getBPOverview = (store: Store<MinaState>): BlockProductionOverviewState => stateSliceAsPromise<BlockProductionOverviewState>(store, condition, 'blockProduction', 'overview');
const execute = (callback: () => void) => {
  cy.wait('@epochSummary')
    .url()
    .then((url: string) => {
      if (url.includes('/block-production/overview')) {
        callback();
      }
    });
};
let epochSummaryResponse: BlockProductionEpochPaginationResponse[];

describe('BLOCK PRODUCTION OVERVIEW EPOCH GRAPHS', () => {
  beforeEach(() => {
    cy
      .intercept(/\/epoch\/summary\/\d+\?limit=\d+/, req => {
        req.continue(res => {
          epochSummaryResponse = res.body;
        });
      })
      .as('epochSummary')
      .visit(Cypress.config().baseUrl + '/block-production/overview');
  });

  it('should have correct title', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-toolbar span')
            .then((span: any) => expect(span).contain('Block Production'));
        }
      });
  }));

  it('should have 7 epochs displayed', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-epoch-graphs .epoch')
            .should('have.length', 7);
        }
      });
  }));

  it('should have correct epochs numbers displayed', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-epoch-graphs .epoch .title')
            .then(items => {
              const itemsCount = items.length;
              items.each((index, item) => {
                const reverseIndex = itemsCount - index - 1;
                expect(item.textContent.trim()).equals((reverseIndex === itemsCount - 1 ? 'Epoch ' : '') + epochSummaryResponse[reverseIndex].epoch_number.toString());
              });
            });
        }
      });
  }));

  it('should have correct windows heights displayed', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-epoch-graphs .epoch')
            .each((epoch, index) => {
              const epochResponse = epochSummaryResponse.reverse()[index];
              if (epochResponse.sub_windows.length) {
                cy.wrap(epoch)
                  .find('.positive > div')
                  .each((window, windowIndex) => {
                    const windowResponse = epochResponse.sub_windows[windowIndex];
                    if (windowResponse.canonical > 0) {
                      expect(window).not.have.class('future');
                      expect(window).not.have.css('height', '0px');
                    } else if (windowResponse.future_rights > 0) {
                      expect(window).have.class('future');
                      expect(window).not.have.css('height', '0px');
                    } else {
                      expect(window).have.css('height', '0px');
                    }
                  })
                  .wrap(epoch)
                  .find('.negative .bar > div:first-child')
                  .each((window, windowIndex) => {
                    const windowResponse = epochResponse.sub_windows[windowIndex];
                    if (windowResponse.orphaned > 0) {
                      expect(window).not.have.css('height', '0px');
                    } else {
                      expect(window).have.css('height', '0px');
                    }
                  })
                  .wrap(epoch)
                  .find('.negative .bar > div:last-child')
                  .each((window, windowIndex) => {
                    const windowResponse = epochResponse.sub_windows[windowIndex];
                    if (windowResponse.missed > 0) {
                      expect(window).not.have.css('height', '0px');
                    } else {
                      expect(window).have.css('height', '0px');
                    }
                  });
              }
            });
        }
      });
  }));

  it('should show correct position of active window', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-epoch-graphs .active-epoch .overlay')
            .should('have.class', 'bg-selected-container')
            .then(overlay => {
              const right = overlay[0].getBoundingClientRect().right;
              const activeEpoch = state.activeEpochNumber;
              const activeWindow = epochSummaryResponse.find(epoch => epoch.epoch_number === activeEpoch).sub_windows.find(window => window.is_current);
              const indexOfActiveWindow = epochSummaryResponse.find(epoch => epoch.epoch_number === activeEpoch).sub_windows.indexOf(activeWindow);
              cy.get(`mina-block-production-overview-epoch-graphs .active-epoch .positive > div:nth-child(${indexOfActiveWindow + 1})`)
                .then(activeWindow => {
                  expect(activeWindow[0].getBoundingClientRect().right).to.be.closeTo(right, 2);
                });
            });

        }
      });
  }));
});
