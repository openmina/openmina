import { Injectable } from '@angular/core';
import { BlockProductionOverviewService } from '@block-production/overview/block-production-overview.service';
import { map, Observable } from 'rxjs';
import {
  BlockProductionWonSlotsDiscardReason,
  BlockProductionWonSlotsSlot,
  BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { BlockProductionModule } from '@block-production/block-production.module';
import {
  BlockProductionOverviewEpochDetails,
} from '@shared/types/block-production/overview/block-production-overview-epoch-details.type';
import { nanOrElse, ONE_MILLION } from '@openmina/shared';
import { getTimeDiff } from '@shared/helpers/date.helper';
import { HttpClient } from '@angular/common/http';

@Injectable({
  providedIn: BlockProductionModule,
})
export class BlockProductionWonSlotsService {

  constructor(private service: BlockProductionOverviewService,
              private http: HttpClient) {
  }

  getActiveEpoch(): Observable<BlockProductionOverviewEpochDetails> {
    return this.service.getEpochDetails();
  }

  //
  // getSlots(epoch: number): Observable<BlockProductionWonSlotsSlot[]> {
  //   return this.service.getSlots(epoch).pipe(
  //     map((slots) => {
  //       return slots
  //         .filter(s => s.canonical || s.missed || s.orphaned || s.futureRights || s.active)
  //         .map((slot) => ({
  //           ...slot,
  //           message: this.getMessage(slot),
  //           age: this.calculateTimeAgo(slot),
  //           snarkFees: Math.floor(Math.random() * 100),
  //           transactionsTotal: Math.floor(Math.random() * 100),
  //           coinbaseRewards: Math.floor(Math.random() * 100),
  //           txFeesRewards: Math.floor(Math.random() * 100),
  //           vrfValueWithThreshold: Math.floor(Math.random() * 100),
  //           creatingStagedLedgerDiffElapsedTime: Math.floor(Math.random() * 100),
  //           creatingBlockProofElapsedTime: Math.floor(Math.random() * 100),
  //           applyingBlockElapsedTime: Math.floor(Math.random() * 100),
  //           broadcastedBlockElapsedTime: Math.floor(Math.random() * 100),
  //         }));
  //     }),
  //   );
  // }

  getSlots(): Observable<BlockProductionWonSlotsSlot[]> {
    return this.http.get<WonSlotResponse>('http://65.109.110.75:11010/stats/block_producer')
      // return of({
      //   'attempts': [{
      //     'won_slot': {
      //       'slot_time': 1716753481000000000,
      //       'global_slot': 54839,
      //       'epoch': 7,
      //       'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //       'value_with_threshold': [0.0011658520089635517, 0.0027039420629577974],
      //     },
      //     'block': {
      //       'hash': '3NLFi4aEzpz1o58RyNYFyc38m4L5RTNpoCJ1fDvnc7jmEeKqAYXr',
      //       'height': 32415,
      //       'transactions': { 'payments': 2, 'delegations': 0, 'zkapps': 1 },
      //       'coinbase': 0,
      //       'fees': 0,
      //       'snark_fees': 0,
      //     },
      //     'times': {
      //       'scheduled': 1716724539185197872,
      //       'staged_ledger_diff_create_start': 1716753481007937392,
      //       'staged_ledger_diff_create_end': 1716753481110198070,
      //       'produced': 1716753481110222193,
      //       'proof_create_start': 1716753481111317798,
      //       'proof_create_end': null,
      //       'block_apply_start': null,
      //       'block_apply_end': null,
      //       'discarded': null,
      //     },
      //     'status': 'ProofCreatePending',
      //   }, {
      //     'won_slot': {
      //       'slot_time': 1716911161000000000,
      //       'global_slot': 55715,
      //       'epoch': 7,
      //       'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //       'value_with_threshold': [0.0010286996255322537, 0.0027039420629577974],
      //     },
      //     'block': null,
      //     'times': {
      //       'scheduled': 1716753496902575613,
      //       'staged_ledger_diff_create_start': null,
      //       'staged_ledger_diff_create_end': null,
      //       'produced': null,
      //       'proof_create_start': null,
      //       'proof_create_end': null,
      //       'block_apply_start': null,
      //       'block_apply_end': null,
      //       'discarded': null,
      //     },
      //     'status': 'Scheduled',
      //   }],
      //   'future_won_slots': [{
      //     'slot_time': 1716938161000000000,
      //     'global_slot': 55865,
      //     'epoch': 7,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.002236544047945737, 0.0027039420629577974],
      //   }, {
      //     'slot_time': 1716977041000000000,
      //     'global_slot': 56081,
      //     'epoch': 7,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0000566058528236118, 0.0027039420629577974],
      //   }, {
      //     'slot_time': 1717056961000000000,
      //     'global_slot': 56525,
      //     'epoch': 7,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.001227838098840842, 0.0027039420629577974],
      //   }, {
      //     'slot_time': 1717093141000000000,
      //     'global_slot': 56726,
      //     'epoch': 7,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0013454599854690742, 0.0027039420629577974],
      //   }, {
      //     'slot_time': 1717144261000000000,
      //     'global_slot': 57010,
      //     'epoch': 7,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.00023698420290702414, 0.0027039420629577974],
      //   }, {
      //     'slot_time': 1717266841000000000,
      //     'global_slot': 57691,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0023906140447840177, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717291861000000000,
      //     'global_slot': 57830,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0013813847118976153, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717312561000000000,
      //     'global_slot': 57945,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0008566241304215388, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717325521000000000,
      //     'global_slot': 58017,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0011020372218478133, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717370701000000000,
      //     'global_slot': 58268,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0019381773953531737, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717377181000000000,
      //     'global_slot': 58304,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.000815976005158431, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717387621000000000,
      //     'global_slot': 58362,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0002676889437583526, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717501381000000000,
      //     'global_slot': 58994,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0006334875594822137, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717516501000000000,
      //     'global_slot': 59078,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0013311406556081385, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717587241000000000,
      //     'global_slot': 59471,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0023410820819993136, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717643401000000000,
      //     'global_slot': 59783,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0006114640436674149, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717650781000000000,
      //     'global_slot': 59824,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.000915561936191267, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717657261000000000,
      //     'global_slot': 59860,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0016385582964355792, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717726921000000000,
      //     'global_slot': 60247,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.00265915452828488, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717830061000000000,
      //     'global_slot': 60820,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0023476414638817626, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717847701000000000,
      //     'global_slot': 60918,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0006264297372377878, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717872901000000000,
      //     'global_slot': 61058,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0009573060237176475, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717890361000000000,
      //     'global_slot': 61155,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.002144163453869725, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1717989721000000000,
      //     'global_slot': 61707,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.002265010344681852, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1718074861000000000,
      //     'global_slot': 62180,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0016454544345777793, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1718117701000000000,
      //     'global_slot': 62418,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0019403103896273853, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1718122201000000000,
      //     'global_slot': 62443,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.001917307088117943, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1718275381000000000,
      //     'global_slot': 63294,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.0010936328551288577, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1718317501000000000,
      //     'global_slot': 63528,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.002694265606320862, 0.0027118530172575996],
      //   }, {
      //     'slot_time': 1718376181000000000,
      //     'global_slot': 63854,
      //     'epoch': 8,
      //     'delegator': ['B62qpdPtz7QSTcPLkDuSdGGv6VkdhG5Gy2NLFBLfkyfR6K3KSfviW4Y', 172],
      //     'value_with_threshold': [0.001541071560515784, 0.0027118530172575996],
      //   }],
      // })
      .pipe(
        map(({ attempts, future_won_slots }: WonSlotResponse) => {
          const attemptsSlots = attempts.map((attempt: Attempt) => {
            attempt.active = attempt.status !== BlockProductionWonSlotsStatus.Discarded && attempt.status !== BlockProductionWonSlotsStatus.Committed;
            attempt.won_slot.slot_time = attempt.won_slot.slot_time / ONE_MILLION;
            return {
              message: this.getMessage(attempt),
              age: this.calculateTimeAgo(attempt),
              slotTime: attempt.won_slot.slot_time,
              globalSlot: attempt.won_slot.global_slot,
              vrfValueWithThreshold: attempt.won_slot.value_with_threshold,
              active: attempt.active,

              height: attempt.block?.height,
              hash: attempt.block?.hash,
              transactionsTotal: nanOrElse(attempt.block?.transactions.payments + attempt.block?.transactions.delegations + attempt.block?.transactions.zkapps, 0),
              payments: nanOrElse(attempt.block?.transactions.payments, 0),
              delegations: nanOrElse(attempt.block?.transactions.delegations, 0),
              zkapps: nanOrElse(attempt.block?.transactions.zkapps, 0),
              snarkFees: attempt.block?.snark_fees,
              coinbaseRewards: attempt.block?.coinbase,
              txFeesRewards: attempt.block?.fees,

              status: attempt.status,
              discardReason: attempt.discard_reason,

              times: {
                scheduled: attempt.times.scheduled,
                stagedLedgerDiffCreate: !attempt.times.staged_ledger_diff_create_end || !attempt.times.staged_ledger_diff_create_start
                  ? null : (attempt.times.staged_ledger_diff_create_end - attempt.times.staged_ledger_diff_create_start) / ONE_MILLION,
                produced: !attempt.times.produced || !attempt.times.staged_ledger_diff_create_end
                  ? null : (attempt.times.produced - attempt.times.staged_ledger_diff_create_end) / ONE_MILLION,
                proofCreate: !attempt.times.proof_create_end || !attempt.times.proof_create_start
                  ? null : (attempt.times.proof_create_end - attempt.times.proof_create_start) / ONE_MILLION,
                blockApply: !attempt.times.block_apply_end || !attempt.times.block_apply_start
                  ? null : (attempt.times.block_apply_end - attempt.times.block_apply_start) / ONE_MILLION,
                discarded: attempt.times.discarded,
                committed: attempt.times.committed,
              },
            } as BlockProductionWonSlotsSlot;
          });

          const futureWonSlots = future_won_slots.map((slot: WonSlot) => {
            slot.slot_time = slot.slot_time / ONE_MILLION;
            return {
              message: 'Upcoming Won Slot',
              age: this.calculateTimeAgo({ won_slot: slot }),
              slotTime: slot.slot_time,
              globalSlot: slot.global_slot,
              vrfValueWithThreshold: slot.value_with_threshold,
              active: false,
            } as BlockProductionWonSlotsSlot;
          });

          return [...attemptsSlots, ...futureWonSlots];
        }),
      );
  }

  private getMessage(attempt: Attempt): string {
    if (attempt.active) {
      return 'Currently Producing';
    }
    // if (block.canonical) {
    //   return 'Accepted Block';
    // } else if (block.orphaned) {
    //   return 'Rejected Block';
    // } else if (block.missed) {
    //   return 'Missed Block';
    // }
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
}

interface WonSlotResponse {
  attempts: Attempt[];
  future_won_slots: WonSlot[];
}

interface Attempt {
  won_slot: WonSlot;
  block?: Block;
  times: Times;
  status: BlockProductionWonSlotsStatus;
  active?: boolean;
  discard_reason?: BlockProductionWonSlotsDiscardReason;
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
