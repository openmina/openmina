import { WonSlotResponse } from '@block-production/won-slots/block-production-won-slots.service';
import {
  BlockProductionWonSlotsDiscardReason,
  BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { cyIsSubFeatureEnabled, stateSliceAsPromise } from '../../../support/commands';
import { AppState } from '@app/app.state';

const getAppState = (store: Store<MinaState>): AppState => stateSliceAsPromise<AppState>(store, () => true, 'app');

let response: WonSlotResponse;

describe('BLOCK PRODUCTION WON SLOTS APIS', () => {
  beforeEach(() => {
    console.log('beforeEach');
    cy
      .visit(Cypress.config().baseUrl)
      .window()
      .its('store')
      .then(getAppState)
      .then((state: AppState) => {
        cy.window()
          .its('config')
          .then((config: any) => {
            if (cyIsSubFeatureEnabled(state.activeNode, 'block-production', 'won-slots', config.globalConfig)) {
              cy
                .intercept('/stats/block_producer', (req) => {
                  req.continue(res => {
                    response = res.body;
                  });
                })
                .as('request');
            }
          });
      });
  });

  it('validate block producer attempts json data', () => {
    cy
      .visit(Cypress.config().baseUrl + '/block-production/won-slots')
      .wait('@request')
      .url()
      .then((url: string) => {
        if (url.includes('/block-production/won-slots')) {
          expect(response).to.exist;
          expect(response.current_global_slot).to.exist;
          expect(response.current_time).to.exist;
          expect(response.epoch_start).to.exist;
          expect(response.epoch_end).to.exist;
          expect(response.attempts).to.exist;
          expect(response.future_won_slots).to.exist;

          if (response.attempts.length > 0) {
            const allAssertionsOk = response.attempts.every(attempt =>
              attempt.won_slot !== undefined &&
              attempt.times !== undefined &&
              attempt.status !== undefined &&
              attempt.block !== undefined &&
              Object.values(BlockProductionWonSlotsStatus).includes(attempt.status),
            );

            expect(allAssertionsOk).to.be.true;
            expect(response.attempts.filter(attempt => getActive(attempt)).length <= 1 ? 'foundActiveAttempt' : 'noActiveAttempt').to.equal('foundActiveAttempt');


            const allTimesInNanoseconds = response.attempts.every(attempt => {
              if (attempt.times) {
                return Object.values(attempt.times).filter(Boolean).every(time => time.toString().length === 19);
              }
              return true;
            });

            expect(allTimesInNanoseconds ? 'allTimesInNanoseconds' : 'not allTimesInNanoseconds').to.equal('allTimesInNanoseconds');

            const slotTimes = response.attempts
              .map(attempt => attempt.won_slot.slot_time)
              .reduce((acc, current, index, arr) => {
                return [...acc, arr[index - 1] ? current > arr[index - 1] : true];
              }, []);
            expect(slotTimes.every(Boolean) ? 'slotTimesIncreasing' : 'slotTimesNotIncreasing').to.equal('slotTimesIncreasing');

            const globalSlots = response.attempts
              .map(attempt => attempt.won_slot.global_slot)
              .reduce((acc, current, index, arr) => {
                return [...acc, arr[index - 1] ? current > arr[index - 1] : true];
              }, []);
            expect(globalSlots.every(Boolean) ? 'globalSlotsIncreasing' : 'globalSlotsNotIncreasing').to.equal('globalSlotsIncreasing');

            const discardedAttempts = response.attempts.filter(attempt => attempt.status === BlockProductionWonSlotsStatus.Discarded);
            if (discardedAttempts.length > 0) {
              const discardedReasonsExist = discardedAttempts.every(attempt => {
                const reason = getDiscardReason(attempt);
                return reason !== undefined;
              });
              expect(discardedReasonsExist ? 'discardedReasonsExist' : 'discardedReasonsDoNotExist').to.equal('discardedReasonsExist');
            }

            const orphanedBlocks = response.attempts.filter(attempt => attempt.status === BlockProductionWonSlotsStatus.Orphaned);
            if (orphanedBlocks.length > 0) {
              const orphanedByExists = orphanedBlocks.every(attempt => attempt.orphaned_by !== undefined);
              expect(orphanedByExists ? 'orphanedByExists' : 'orphanedByDoesNotExist').to.equal('orphanedByExists');
            }

            const canonicalBlocks = response.attempts.filter(attempt => attempt.status === BlockProductionWonSlotsStatus.Canonical);
            if (canonicalBlocks.length > 0) {
              const lastObservedConfirmationsExist = canonicalBlocks.every(attempt => attempt.last_observed_confirmations !== undefined);
              expect(lastObservedConfirmationsExist ? 'lastObservedConfirmationsExist' : 'lastObservedConfirmationsDoNotExist').to.equal('lastObservedConfirmationsExist');
            }
          }
        }
      });
  });

  it('validate block producer future won slots json data', () => {
    cy
      .visit(Cypress.config().baseUrl + '/block-production/won-slots')
      .wait('@request')
      .url()
      .then((url: string) => {
        if (url.includes('/block-production/won-slots')) {
          expect(response).to.exist;
          expect(response.current_global_slot).to.exist;
          expect(response.current_time).to.exist;
          expect(response.epoch_start).to.exist;
          expect(response.epoch_end).to.exist;
          expect(response.attempts).to.exist;
          expect(response.future_won_slots).to.exist;

          if (response.future_won_slots.length > 0) {
            const allAssertionsOk = response.future_won_slots.every(slot =>
              slot.slot_time !== undefined &&
              slot.global_slot !== undefined &&
              slot.value_with_threshold !== undefined,
            );

            expect(allAssertionsOk).to.be.true;

            const slotTimes = response.future_won_slots
              .map(attempt => attempt.slot_time)
              .reduce((acc, current, index, arr) => {
                return [...acc, arr[index - 1] ? current > arr[index - 1] : true];
              }, []);
            expect(slotTimes.every(Boolean) ? 'slotTimesIncreasing' : 'slotTimesNotIncreasing').to.equal('slotTimesIncreasing');

            const globalSlots = response.future_won_slots
              .map(attempt => attempt.global_slot)
              .reduce((acc, current, index, arr) => {
                return [...acc, arr[index - 1] ? current > arr[index - 1] : true];
              }, []);
            expect(globalSlots.every(Boolean) ? 'globalSlotsIncreasing' : 'globalSlotsNotIncreasing').to.equal('globalSlotsIncreasing');

            const hasThresholdWithValue = response.future_won_slots.every(slot => slot.value_with_threshold.length === 2);
            expect(hasThresholdWithValue ? 'hasThresholdWithValue' : 'noThresholdWithValue').to.equal('hasThresholdWithValue');

          }
        }
      });
  });
});


function getActive(attempt: WonSlotResponse['attempts'][0]): boolean {
  const slotTime = attempt.won_slot.slot_time;
  const now = Date.now();
  return slotTime <= now && (now < 3 * 60 * 1000 + slotTime) && !attempt.times?.discarded;
}

function getDiscardReason(attempt: WonSlotResponse['attempts'][0]): BlockProductionWonSlotsDiscardReason {
  let reason;
  Object.keys(attempt).forEach((key) => {
    if (key in BlockProductionWonSlotsDiscardReason) {
      reason = key;
    }
  });
  return reason;
}
