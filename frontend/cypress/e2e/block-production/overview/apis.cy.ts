import {
  AllStatsResponse,
  BlockProductionEpochPaginationResponse,
  SlotResponse,
} from '@block-production/overview/block-production-overview.service';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { cyIsSubFeatureEnabled, stateSliceAsPromise } from '../../../support/commands';
import { AppState } from '@app/app.state';


const getAppState = (store: Store<MinaState>): AppState => stateSliceAsPromise<AppState>(store, () => true, 'app');
const getConfig = () => cy.window().its('config');
const execute = (callback: () => void) => {
  cy.visit(Cypress.config().baseUrl)
    .window()
    .its('store')
    .then(getAppState)
    .then((state: AppState) => {
      getConfig().then((config: any) => {
        if (cyIsSubFeatureEnabled(state.activeNode, 'block-production', 'overview', config.globalConfig)) {
          cy.wait('@slotsRequest')
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

describe('BLOCK PRODUCTION OVERVIEW APIs', () => {
  it('validate epoch details json data', () => execute(() => {
    let epoch: BlockProductionEpochPaginationResponse;
    let epochNumber: number | string;
    cy
      .intercept(/\/epoch\/summary\/(?!.*\?limit=\d+)(latest|\d+)/, (req) => {
        req.continue((res: any) => {
          epochNumber = res.url.split('/')[res.url.split('/').length - 1];
          if (!Array.isArray(res.body)) {
            res.body = [res.body];
          }
          epoch = res.body[0];
        });
      })
      .as('epochDetailsRequest')
      .visit(Cypress.config().baseUrl + '/block-production/overview')
      .wait('@epochDetailsRequest')
      .url()
      .then((url: string) => {
        if (url.includes('/block-production/overview')) {
          expect(epoch).to.exist;
          expect(epoch.epoch_number).to.exist;
          expect(epoch.balance_delegated).to.exist;
          expect(epoch.balance_producer).to.exist;
          expect(epoch.balance_staked).to.exist;
          if (epoch.summary) {
            expect(epoch.summary.won_slots).to.exist;
            expect(epoch.summary.canonical).to.exist;
            expect(epoch.summary.orphaned).to.exist;
            expect(epoch.summary.missed).to.exist;
            expect(epoch.summary.future_rights).to.exist;
            expect(epoch.summary.slot_start).to.exist;
            expect(epoch.summary.slot_end).to.exist;
            expect(epoch.summary.expected_rewards).to.exist;
            expect(epoch.summary.earned_rewards).to.exist;
          }

          expect(epochNumber).to.satisfy((epochNumber: number | string) => epochNumber === 'latest' || Number.isInteger(Number(epochNumber)));
          expect(epoch.balance_delegated).to.be.a('string');
          expect(epoch.balance_delegated).to.match(/^[0-9]+$/);
          expect(epoch.balance_producer).to.be.a('string');
          expect(epoch.balance_producer).to.match(/^[0-9]+$/);
          expect(epoch.balance_staked).to.be.a('string');
          expect(epoch.balance_staked).to.match(/^[0-9]+$/);
          expect(epoch.sub_windows.length).to.satisfy((length: number) => length === 0 || length === 15);
          epoch.sub_windows.forEach((subWindow, i) => {
            if (subWindow.canonical) {
              expect(subWindow.earned_rewards).to.not.equal('0');
            } else {
              expect(subWindow.earned_rewards).to.equal('0');
            }
            expect(subWindow.orphaned + subWindow.missed).to.be.lte(subWindow.max);
            expect(subWindow.canonical).to.be.at.most(subWindow.won_slots);
            if (i > 0) {
              expect(subWindow.slot_start).to.equal(epoch.sub_windows[i - 1].slot_end + 1);
            }
            if (i < epoch.sub_windows.length - 1) {
              expect(subWindow.slot_end).to.equal(epoch.sub_windows[i + 1].slot_start - 1);
            }
            expect(subWindow.slot_end - subWindow.slot_start).to.equal(475);
            expect(Object.values(subWindow)).to.not.include(null);
          });
          expect(epoch.sub_windows.filter(w => w.is_current)).to.have.lengthOf.at.most(1);

          if (epoch.summary) {
            expect(epoch.summary.won_slots).to.be.a('number');
            expect(epoch.summary.canonical).to.be.a('number');
            expect(epoch.summary.orphaned).to.be.a('number');
            expect(epoch.summary.missed).to.be.a('number');
            expect(epoch.summary.future_rights).to.be.a('number');
            expect(epoch.summary.slot_start).to.be.a('number');
            expect(epoch.summary.slot_end).to.be.a('number');
            expect(epoch.summary.expected_rewards).to.be.a('string');
            expect(epoch.summary.earned_rewards).to.be.a('string');
            expect(epoch.summary.canonical).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.canonical, 0));
            expect(epoch.summary.orphaned).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.orphaned, 0));
            expect(epoch.summary.missed).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.missed, 0));
            expect(epoch.summary.future_rights).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.future_rights, 0));
            expect(epoch.summary.won_slots).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.won_slots, 0));
            expect(epoch.summary.slot_start).to.equal(epoch.sub_windows[0].slot_start);
            expect(epoch.summary.slot_end).to.equal(epoch.sub_windows[epoch.sub_windows.length - 1].slot_end);
          }
        }
      });
  }));

  it('validate all stats json data', () => execute(() => {
    let allStats: AllStatsResponse;
    let epochList: BlockProductionEpochPaginationResponse[];
    cy
      .intercept(/\/epoch\/summary\/\d+\?limit=\d+/, req => {
        req.continue(res => {
          epochList = res.body;
        });
      })
      .as('epochList')
      .intercept('/summary', (req) => {
        req.continue((res: any) => {
          allStats = res.body;
        });
      })
      .as('allStatsRequest')
      .visit(Cypress.config().baseUrl + '/block-production/overview')
      .wait('@allStatsRequest')
      .wait('@epochList')
      .url()
      .then((url: string) => {
        if (url.includes('/block-production/overview')) {
          expect(allStats).to.not.be.undefined;
          expect(allStats.won_slots).to.exist;
          expect(allStats.canonical).to.exist;
          expect(allStats.orphaned).to.exist;
          expect(allStats.missed).to.exist;
          expect(allStats.future_rights).to.exist;
          expect(allStats.expected_rewards).to.exist;
          expect(allStats.earned_rewards).to.exist;
          expect(allStats.won_slots).to.be.a('number');
          expect(allStats.canonical).to.be.a('number');
          expect(allStats.orphaned).to.be.a('number');
          expect(allStats.missed).to.be.a('number');
          expect(allStats.future_rights).to.be.a('number');
          expect(allStats.expected_rewards).to.be.a('string');
          expect(allStats.earned_rewards).to.be.a('string');
          expect(Object.values(allStats)).to.not.include(null);
          expect(allStats.canonical).to.be.lte(allStats.won_slots);
          expect(allStats.canonical).to.be.at.least(epochList.reduce((acc, e) => acc + (e.summary?.canonical || 0), 0));
          expect(allStats.orphaned).to.be.at.least(epochList.reduce((acc, e) => acc + (e.summary?.orphaned || 0), 0));
          expect(allStats.missed).to.be.at.least(epochList.reduce((acc, e) => acc + (e.summary?.missed || 0), 0));
          expect(allStats.future_rights).to.be.at.least(epochList.reduce((acc, e) => acc + (e.summary?.future_rights || 0), 0));
          expect(allStats.won_slots).to.be.at.least(epochList.reduce((acc, e) => acc + (e.summary?.won_slots || 0), 0));
          expect(Number(allStats.earned_rewards)).to.be.at.least(epochList.reduce((acc, e) => acc + Number(e.summary?.earned_rewards || 0), 0));
          expect(Number(allStats.expected_rewards)).to.be.at.least(epochList.reduce((acc, e) => acc + Number(e.summary?.expected_rewards || 0), 0));
        }
      });
  }));

  it('validate epoch list json data', () => execute(() => {
    let epochList: BlockProductionEpochPaginationResponse[];
    let askedEpochs: number;
    let lastAskedEpoch: number;
    cy
      .intercept(/\/epoch\/summary\/\d+\?limit=\d+/, req => {
        req.continue((res: any) => {
          askedEpochs = Number(res.url.split('=')[1]);
          lastAskedEpoch = Number(res.url.split('/')[res.url.split('/').length - 1].split('?limit')[0]);
          epochList = res.body;
        });
      })
      .as('epochList')
      .visit(Cypress.config().baseUrl + '/block-production/overview')
      .wait('@epochList')
      .url()
      .then((url: string) => {
        if (url.includes('/block-production/overview')) {
          expect(epochList).to.not.be.undefined;
          expect(askedEpochs).to.not.be.undefined;
          expect(lastAskedEpoch).to.not.be.undefined;
          expect(epochList).to.have.lengthOf(askedEpochs);
          expect(epochList[0].epoch_number).to.equal(lastAskedEpoch);
          expect(epochList[epochList.length - 1].epoch_number).to.equal(lastAskedEpoch - askedEpochs + 1);

          epochList.forEach((epoch, i) => {
            expect(epoch.balance_delegated).to.be.a('string');
            expect(epoch.balance_delegated).to.match(/^[0-9]+$/);
            expect(epoch.balance_producer).to.be.a('string');
            expect(epoch.balance_producer).to.match(/^[0-9]+$/);
            expect(epoch.balance_staked).to.be.a('string');
            expect(epoch.balance_staked).to.match(/^[0-9]+$/);
            expect(epoch.sub_windows.length).to.satisfy((length: number) => length === 0 || length === 15);
            expect(epoch.sub_windows.filter(w => w.is_current)).to.have.lengthOf.at.most(1);

            if (epoch.summary) {
              expect(epoch.summary.won_slots).to.be.a('number');
              expect(epoch.summary.canonical).to.be.a('number');
              expect(epoch.summary.orphaned).to.be.a('number');
              expect(epoch.summary.missed).to.be.a('number');
              expect(epoch.summary.future_rights).to.be.a('number');
              expect(epoch.summary.slot_start).to.be.a('number');
              expect(epoch.summary.slot_end).to.be.a('number');
              expect(epoch.summary.expected_rewards).to.be.a('string');
              expect(epoch.summary.earned_rewards).to.be.a('string');
              expect(epoch.summary.canonical).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.canonical, 0));
              expect(epoch.summary.orphaned).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.orphaned, 0));
              expect(epoch.summary.missed).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.missed, 0));
              expect(epoch.summary.future_rights).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.future_rights, 0));
              expect(epoch.summary.won_slots).to.equal(epoch.sub_windows.reduce((acc, w) => acc + w.won_slots, 0));
              expect(epoch.summary.slot_start).to.equal(epoch.sub_windows[0].slot_start);
              expect(epoch.summary.slot_end).to.equal(epoch.sub_windows[epoch.sub_windows.length - 1].slot_end);
            }
          });

          epochList = epochList.reverse();
          for (let i = 0; i < epochList.length - 1; i++) {
            if (epochList[i].sub_windows[epochList[i].sub_windows.length - 1] && epochList[i + 1].sub_windows[0]) {
              expect(epochList[i].sub_windows[epochList[i].sub_windows.length - 1].slot_end + 1).to.equal(epochList[i + 1].sub_windows[0].slot_start);
            }
          }
        }
      });
  }));

  it('validate slots json data', () => execute(() => {
    let slotsResponse: SlotResponse[];
    let askedEpochs: number;
    cy
      .intercept(/\/epoch\/\d+/, (req) => {
        req.continue((res: any) => {
          askedEpochs = Number(res.url.split('/')[res.url.split('/').length - 1]);
          slotsResponse = res.body;
        });
      })
      .as('slotsRequest')
      .visit(Cypress.config().baseUrl + '/block-production/overview')
      .wait('@slotsRequest')
      .url()
      .then((url: string) => {
        if (url.includes('/block-production/overview')) {
          expect(slotsResponse).to.not.be.undefined;
          expect(askedEpochs).to.not.be.undefined;
          expect(slotsResponse).to.have.lengthOf(7140);
          const areSlotsValid = slotsResponse.every(slot => {
            return slot.slot >= 0
              && slot.global_slot >= 0
              && slot.timestamp >= 0
              && (
                ['Empty', 'ToBeProduced', 'Orphaned', 'OrphanedPending', 'Canonical', 'CanonicalPending', 'Foreign', 'Missed'].includes(slot.block_status)
              )
              && (
                slot.state_hash?.startsWith('3N') || slot.state_hash === null
              ) && (
                typeof slot.height === 'number' || slot.state_hash === null
              ) && typeof slot.is_current_slot === 'boolean';
          });
          expect(areSlotsValid ? 'areSlotsValid' : 'areSlotsInvalid').to.equal('areSlotsValid');
          const areGlobalSlotsIncreasing = slotsResponse.every((slot, i) => {
            return i === 0 || slot.global_slot === slotsResponse[i - 1].global_slot + 1;
          });
          expect(areGlobalSlotsIncreasing).to.be.true;
          const areSlotsIncreasing = slotsResponse.every((slot, i) => {
            return i === 0 || slot.slot === slotsResponse[i - 1].slot + 1;
          });
          expect(areSlotsIncreasing).to.be.true;
          expect(slotsResponse.filter(s => s.is_current_slot).length).to.be.at.most(1);

          const areTimesIncreasingWith3Minutes = slotsResponse.every((slot, i) => {
            return i === 0 || slot.timestamp === slotsResponse[i - 1].timestamp + 180;
          });
          expect(areTimesIncreasingWith3Minutes).to.be.true;
        }
      });
  }));
});

