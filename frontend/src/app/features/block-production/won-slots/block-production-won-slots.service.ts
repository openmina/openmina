import { Injectable } from '@angular/core';
import { map, Observable } from 'rxjs';
import {
  BlockProductionWonSlotsDiscardReason,
  BlockProductionWonSlotsSlot,
  BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { BlockProductionModule } from '@block-production/block-production.module';
import { hasValue, nanOrElse, ONE_BILLION, ONE_MILLION } from '@openmina/shared';
import { getTimeDiff } from '@shared/helpers/date.helper';
import { RustService } from '@core/services/rust.service';
import {
  BlockProductionWonSlotsEpoch,
} from '@shared/types/block-production/won-slots/block-production-won-slots-epoch.type';

@Injectable({
  providedIn: BlockProductionModule,
})
export class BlockProductionWonSlotsService {

  constructor(private rust: RustService) { }

  getSlots(): Observable<{ slots: BlockProductionWonSlotsSlot[], epoch: BlockProductionWonSlotsEpoch }> {
    return this.rust.get<WonSlotResponse>('/stats/block_producer')
      .pipe(
        map((response: WonSlotResponse) => {
          if (!response) {
            throw new Error('Empty response from /stats/block_producer');
          }
          const attemptsSlots = response.attempts.map((attempt: Attempt) => {
            attempt.won_slot.slot_time = Math.floor(attempt.won_slot.slot_time / ONE_MILLION); // converted to milliseconds
            attempt.active = BlockProductionWonSlotsService.getActive(attempt);
            let slot = {
              epoch: attempt.won_slot.epoch,
              message: this.getMessage(attempt),
              age: this.calculateTimeAgo(attempt),
              slotTime: attempt.won_slot.slot_time,
              globalSlot: attempt.won_slot.global_slot,
              slotInEpoch: attempt.won_slot.global_slot % 7140,
              vrfValueWithThreshold: attempt.won_slot.value_with_threshold,
              active: attempt.active,

              height: attempt.block?.height,
              hash: attempt.block?.hash,
              transactionsTotal: nanOrElse(attempt.block?.transactions.payments + attempt.block?.transactions.delegations + attempt.block?.transactions.zkapps, 0),
              payments: nanOrElse(attempt.block?.transactions.payments, 0),
              delegations: nanOrElse(attempt.block?.transactions.delegations, 0),
              zkapps: nanOrElse(attempt.block?.transactions.zkapps, 0),
              completedWorksCount: nanOrElse(attempt.block?.completed_works_count, 0),
              snarkFees: attempt.block ? attempt.block.snark_fees / ONE_BILLION : undefined,
              coinbaseRewards: attempt.block ? attempt.block.coinbase / ONE_BILLION : undefined,
              txFeesRewards: attempt.block ? attempt.block.fees / ONE_BILLION : undefined,

              status: attempt.status,
              discardReason: this.getDiscardReason(attempt),
              lastObservedConfirmations: attempt.last_observed_confirmations,
              orphanedBy: attempt.orphaned_by,

              times: {
                scheduled: attempt.times.scheduled,
                stagedLedgerDiffCreate: !attempt.times.staged_ledger_diff_create_end || !attempt.times.staged_ledger_diff_create_start
                  ? null : (attempt.times.staged_ledger_diff_create_end - attempt.times.staged_ledger_diff_create_start) / ONE_BILLION,
                produced: !attempt.times.produced || !attempt.times.staged_ledger_diff_create_end
                  ? null : (attempt.times.produced - attempt.times.staged_ledger_diff_create_end) / ONE_BILLION,
                proofCreate: !attempt.times.proof_create_end || !attempt.times.proof_create_start
                  ? null : (attempt.times.proof_create_end - attempt.times.proof_create_start) / ONE_BILLION,
                blockApply: !attempt.times.block_apply_end || !attempt.times.block_apply_start
                  ? null : (attempt.times.block_apply_end - attempt.times.block_apply_start) / ONE_BILLION,
                discarded: attempt.times.discarded || null,
                committed: attempt.times.committed || null,
                stagedLedgerDiffCreateEnd: attempt.times.staged_ledger_diff_create_end,
                producedEnd: attempt.times.produced,
                proofCreateEnd: attempt.times.proof_create_end,
                blockApplyEnd: attempt.times.block_apply_end,
              },
            } as BlockProductionWonSlotsSlot;

            slot.percentage = slot.active
              ? [
              slot.times?.stagedLedgerDiffCreate,
              slot.times?.produced,
              slot.times?.proofCreate,
              slot.times?.blockApply,
              slot.times?.committed,
            ].filter(t => hasValue(t)).length * 20
              : undefined;
            return slot;
          });

          const futureWonSlots = response.future_won_slots.map((slot: WonSlot) => {
            slot.slot_time = Math.floor(slot.slot_time / ONE_MILLION);
            return {
              message: 'Upcoming Won Slot',
              age: this.calculateTimeAgo({ won_slot: slot }),
              slotTime: slot.slot_time,
              globalSlot: slot.global_slot,
              vrfValueWithThreshold: slot.value_with_threshold,
              active: false,
            } as BlockProductionWonSlotsSlot;
          });

          return {
            slots: [...attemptsSlots, ...futureWonSlots],
            epoch: {
              start: response.epoch_start,
              end: response.epoch_end,
              currentGlobalSlot: response.current_global_slot,
              currentTime: response.current_time,
            },
          };
        }),
      );
  }

  private static getActive(attempt: Attempt): boolean {
    const slotTime = attempt.won_slot.slot_time;
    const now = Date.now();
    return slotTime <= now && (now < 3 * 60 * 1000 + slotTime) && !attempt.times?.discarded;
  }

  private getMessage(attempt: Attempt): string {
    if (attempt.active) {
      return 'Produced';
    }
    if (attempt.status === BlockProductionWonSlotsStatus.Scheduled) {
      return 'Production Scheduled';
    } else if (attempt.status === BlockProductionWonSlotsStatus.Canonical) {
      return 'Produced Block';
    } else if (attempt.status === BlockProductionWonSlotsStatus.Orphaned) {
      return BlockProductionWonSlotsStatus.Orphaned + ' Block';
    } else if (attempt.status === BlockProductionWonSlotsStatus.Discarded) {
      return BlockProductionWonSlotsStatus.Discarded + ' Block';
    } else if (attempt.status === BlockProductionWonSlotsStatus.Committed) {
      return 'Waiting for Confirmation';
    }
    return 'Upcoming Won Slot';
  }

  private calculateTimeAgo({ active, won_slot }: { active?: boolean; won_slot: WonSlot }): string {
    if (active) {
      return 'Now';
    }

    const { diff, inFuture } = getTimeDiff(won_slot.slot_time);

    if (inFuture) {
      return `in ~ ${diff.replace('<', '')}`;
    } else {
      return `${diff} ago`;
    }
  }

  private getDiscardReason(attempt: Attempt): BlockProductionWonSlotsDiscardReason {
    let reason;
    Object.keys(attempt).forEach((key) => {
      if (key in BlockProductionWonSlotsDiscardReason) {
        reason = key;
      }
    });
    return reason;
  }
}

export interface WonSlotResponse {
  attempts: Attempt[];
  future_won_slots: WonSlot[];
  current_global_slot: number;
  current_time: number;
  epoch_end: number;
  epoch_start: number;
}

interface Attempt {
  won_slot: WonSlot;
  block?: Block;
  times: Times;
  status: BlockProductionWonSlotsStatus;
  active?: boolean;
  last_observed_confirmations?: number;
  orphaned_by?: string;
  BestTipStakingLedgerDifferent?: null;
  BestTipGlobalSlotHigher?: null;
  BestTipSuperior?: null;
}

interface WonSlot {
  slot_time: number;
  global_slot: number;
  epoch: number;
  delegator: Array<string | number>;
  value_with_threshold: number[];
}

interface Block {
  hash: string;
  height: number;
  transactions: Transactions;
  coinbase: number;
  fees: number;
  snark_fees: number;
  completed_works_count: number;
}

interface Transactions {
  payments: number;
  delegations: number;
  zkapps: number;
}

interface Times {
  scheduled: number;
  staged_ledger_diff_create_start?: number;
  staged_ledger_diff_create_end?: number;
  produced?: number;
  proof_create_start?: number;
  proof_create_end: number;
  block_apply_start: number;
  block_apply_end: number;
  discarded: number;
  committed: number;
}
