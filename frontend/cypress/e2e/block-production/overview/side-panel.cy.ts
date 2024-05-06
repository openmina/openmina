import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';
import {
  AllStatsResponse,
  BlockProductionDetailsResponse,
} from '@block-production/overview/block-production-overview.service';

const condition = (state: BlockProductionOverviewState): boolean => state && state.epochs?.length > 0;
const getBPOverview = (store: Store<MinaState>): BlockProductionOverviewState => stateSliceAsPromise<BlockProductionOverviewState>(store, condition, 'blockProduction', 'overview');
const execute = (callback: () => void) => {
  cy
    .wait('@allStatsRequest')
    .wait('@epochDetailsRequest')
    .url()
    .then((url: string) => {
      if (url.includes('/block-production/overview')) {
        callback();
      }
    });
};
let epochDetails: BlockProductionDetailsResponse;
let allStats: AllStatsResponse;

describe('BLOCK PRODUCTION OVERVIEW SIDE PANEL', () => {
  beforeEach(() => {
    cy
      .intercept('/epoch/summary/latest', (req) => {
        req.continue(res => {
          epochDetails = res.body[0];
        });
      })
      .as('epochDetailsRequest')
      .intercept(/\/summary/, req => {
        req.continue(res => {
          allStats = res.body;
        });
      })
      .as('allStatsRequest')
      .visit(Cypress.config().baseUrl + '/block-production/overview');
  });

  it('should show correct slots interval', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-side-panel .tab')
            .should('have.text', 'Epoch ' + epochDetails.epoch_number);
        }
      });
  }));
});

