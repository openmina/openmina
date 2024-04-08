import { Injectable } from '@angular/core';
import { BlockProductionModule } from '@app/features/block-production/block-production.module';
import { delay, map, Observable, of } from 'rxjs';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import {
  BlockProductionOverviewEpochDetails,
} from '@shared/types/block-production/overview/block-production-overview-epoch-details.type';
import { hasValue, lastItem, ONE_BILLION } from '@openmina/shared';
import { RustService } from '@core/services/rust.service';
import { BlockProductionSlot } from '@shared/types/block-production/overview/block-production-overview-slot.type';
import {
  BlockProductionOverviewAllStats,
} from '@shared/types/block-production/overview/block-production-overview-all-stats.type';


@Injectable({
  providedIn: BlockProductionModule,
})
export class BlockProductionOverviewService {

  constructor(private rust: RustService) { }

  private epochs: BlockProductionOverviewEpoch[];// = this.mockAll();

  getEpochDetails(epochNumber: number): Observable<BlockProductionOverviewEpochDetails> {
    return this.rust.get<BlockProductionDetailsResponse | BlockProductionDetailsResponse[]>(`/epoch/summary/${epochNumber ?? 'latest'}`).pipe(
      map((response: BlockProductionDetailsResponse | BlockProductionDetailsResponse[]) => {
        if (Array.isArray(response)) {
          response = response[0];
        }
        if (!response.summary) {
          return {
            epochNumber: response.epoch_number,
            totalSlots: 0,
            wonSlots: 0,
            canonical: 0,
            orphaned: 0,
            missed: 0,
            futureRights: 0,
            slotStart: 0,
            slotEnd: 0,
            expectedRewards: null,
            earnedRewards: null,
            balanceDelegated: null,
            balanceProducer: null,
            balanceStaked: null,
          };
        }
        return {
          epochNumber: response.epoch_number,
          totalSlots: response.summary.canonical + response.summary.orphaned + response.summary.missed + response.summary.future_rights,
          wonSlots: response.summary.won_slots,
          canonical: response.summary.canonical,
          orphaned: response.summary.orphaned,
          missed: response.summary.missed,
          futureRights: response.summary.future_rights,
          slotStart: response.summary.slot_start,
          slotEnd: response.summary.slot_end,
          expectedRewards: new Intl.NumberFormat('en-US', {
            maximumFractionDigits: 2,
            useGrouping: false,
          }).format(Number(response.summary.expected_rewards) / ONE_BILLION),
          earnedRewards: new Intl.NumberFormat('en-US', {
            maximumFractionDigits: 2,
            useGrouping: false,
          }).format(Number(response.summary.earned_rewards) / ONE_BILLION),
          balanceDelegated: new Intl.NumberFormat('en-US', {
            maximumFractionDigits: 2,
            useGrouping: false,
          }).format(Number(response.balance_delegated) / ONE_BILLION),
          balanceProducer: new Intl.NumberFormat('en-US', {
            maximumFractionDigits: 2,
            useGrouping: false,
          }).format(Number(response.balance_producer) / ONE_BILLION),
          balanceStaked: new Intl.NumberFormat('en-US', {
            maximumFractionDigits: 2,
            useGrouping: false,
          }).format(Number(response.balance_staked) / ONE_BILLION),
        };
      }),
    );
  }

  getSlots(epochNumber: number): Observable<BlockProductionSlot[]> {
    // return of(this.getMockEpochDetails())
    return this.rust.get<SlotResponse[]>(`/epoch/${epochNumber ?? 'latest'}`).pipe(
      map((response: SlotResponse[]) => {
        const activeSlotIndex = response.findIndex(slot => slot.is_current_slot);
        return response.map((slot: SlotResponse, i: number) => ({
          slot: slot.slot,
          globalSlot: slot.global_slot,
          height: slot.height,
          time: slot.timestamp,
          finished: i < activeSlotIndex && slot.block_status !== BlockStatus.Empty,
          canonical: slot.block_status === BlockStatus.Canonical || slot.block_status === BlockStatus.CanonicalPending,
          orphaned: slot.block_status === BlockStatus.Orphaned || slot.block_status === BlockStatus.OrphanedPending,
          missed: slot.block_status === BlockStatus.Missed,
          futureRights: slot.block_status === BlockStatus.ToBeProduced,
          active: slot.is_current_slot,
          hash: slot.state_hash,
        } as BlockProductionSlot));
      }),
    );
  }

  getEpochs(epochNumber?: number, limit: number = 5): Observable<BlockProductionOverviewEpoch[]> {
    if (hasValue(epochNumber)) {
      epochNumber = epochNumber + 3;
      epochNumber = Math.max(epochNumber, 6);
      limit = limit + 2;
    } else {
      limit = limit + 2;
    }
    return this.rust.get<BlockProductionEpochPaginationResponse[]>(`/epoch/summary/${epochNumber}?limit=${limit}`).pipe(
      map((response: BlockProductionEpochPaginationResponse[]) =>
        response.reverse().map((epoch: BlockProductionEpochPaginationResponse) => ({
          epochNumber: epoch.epoch_number,
          isLastEpoch: epoch.sub_windows.some(w => w.future_rights > 0),
          finishedWindows: epoch.sub_windows.findIndex(w => w.is_current) + 1,
          windows: epoch.sub_windows.map(w => ({
            start: w.slot_start,
            end: w.slot_end,
            canonical: w.canonical,
            orphaned: w.orphaned,
            missed: w.missed,
            futureRights: w.future_rights,
            interval: [w.slot_start, w.slot_end],
          })),
        })),
      ),
    );
    // return this.getEpochsPage(epochNumber, limit);
  }

  private getEpochsPage(epochNumber: number, limit: number): Observable<BlockProductionOverviewEpoch[]> {
    if (!hasValue(epochNumber) || epochNumber > (lastItem(this.epochs).epochNumber)) {
      epochNumber = lastItem(this.epochs).epochNumber;
    }

    const response: BlockProductionOverviewEpoch[] = [];
    this.epochs.forEach((epoch, index) => {
      if ((epoch.epochNumber > epochNumber - limit && epoch.epochNumber <= epochNumber)) {
        response.push(epoch);
      }
    });

    if (response.length < limit) {
      const missing = limit - response.length;
      this.epochs.slice(limit - missing, limit).forEach((epoch, index) => {
        response.push(epoch);
      });
    }
    return of(response).pipe(delay(300));
  }

  getRewardsAllTimeStats(): Observable<BlockProductionOverviewAllStats> {
    return this.rust.get<AllStatsResponse>('/summary').pipe(
      map((response: AllStatsResponse) => ({
        wonSlots: response.won_slots,
        canonical: response.canonical,
        orphaned: response.orphaned,
        missed: response.missed,
        futureRights: response.future_rights,
        totalSlots: response.canonical + response.orphaned + response.missed + response.future_rights,
        expectedRewards: new Intl.NumberFormat('en-US', {
          maximumFractionDigits: 2,
          useGrouping: false,
        }).format(Number(response.expected_rewards) / ONE_BILLION),
        earnedRewards: new Intl.NumberFormat('en-US', {
          maximumFractionDigits: 2,
          useGrouping: false,
        }).format(Number(response.earned_rewards) / ONE_BILLION),
      })),
    );
  }

  private getMockEpochDetails(): BlockProductionSlot[] {
    // generate slots interval  6732–7300
    // with random values
    // globalSlot is the index starting from 6732
    const slots = [];
    for (let i = 6001; i <= 11000; i++) {
      slots.push({
        slot: 0,
        globalSlot: i,
        height: 0,
        time: Date.now() - Math.floor(Math.random() * 14 * 24 * 60 * 60 * 1000),
        finished: true,
        canonical: Math.random() > 0.95,
        orphaned: Math.random() > 0.95,
        missed: Math.random() > 0.99,
        futureRights: false,
        active: i === 11000,
        hash: '',
      });
    }
    // rest push only slots where only futureRights can be true
    for (let i = 11001; i <= 13140; i++) {
      slots.push({
        slot: 0,
        globalSlot: i,
        height: 0,
        time: Math.floor(Math.random() * 100),
        finished: false,
        canonical: false,
        orphaned: false,
        missed: false,
        futureRights: Math.random() > 0.98,
        active: false,
        hash: '',
      });
    }
    return slots;
  }

  private mockAll(): BlockProductionOverviewEpoch[] {
    const epochs: BlockProductionOverviewEpoch[] = [];
    const totalEpochs = 20;
    for (let i = 0; i < totalEpochs; i++) {
      const epochStart = new Date().setTime(new Date().getTime() - (21420 * (totalEpochs - i)) * 1000 * 60);
      const epochEnd = epochStart + 21420 * 60 * 1000;

      let windows = [];
      // each epoch is divided in 15 windows. one window has time 21420 minutes / 15. 21420 / 15 = 1428
      // 7140 blocks divided by 15 = 476
      for (let j = 0; j <= 14; j++) {
        const windowStart = epochStart + 1428 * 60 * 1000 * (14 - j);
        const windowEnd = windowStart + 1428 * 60 * 1000;
        windows.push({
          start: windowStart,
          end: windowEnd,
          canonical: Math.floor(Math.random() * 238),
          orphaned: Math.floor(Math.random() * 238 / 2),
          missed: Math.floor(Math.random() * 238 / 2),
          futureRights: Math.floor(Math.random() * 238),
          interval: [i * 7140 + j * 238, i * 7140 + j * 238 + 238],
        });
      }

      epochs.push({
        epochNumber: i,
        windows,
        finishedWindows: windows.filter(w => w.canonical || w.orphaned || w.missed).length,
      } as BlockProductionOverviewEpoch);
    }

    epochs[epochs.length - 1].windows.slice(4).forEach(w => {
      w.canonical = 0;
      w.missed = 0;
      w.orphaned = 0;
    });
    epochs[epochs.length - 1].finishedWindows = epochs[epochs.length - 1].windows.filter(w => w.canonical || w.orphaned || w.missed).length;
    epochs.find(e => e.windows.some(w => w.canonical === 0 && w.missed === 0 && w.orphaned === 0)).isLastEpoch = true;

    return epochs;
  }
}

interface BlockProductionDetailsResponse {
  epoch_number: number;
  balance_delegated: string;
  balance_producer: string;
  balance_staked: string;
  summary: {
    won_slots: number;
    canonical: number;
    orphaned: number;
    missed: number;
    future_rights: number;
    slot_start: number;
    slot_end: number;
    expected_rewards: string;
    earned_rewards: string;
  } | null;
}

interface BlockProductionEpochPaginationResponse {
  epoch_number: number;
  summary: {
    max: number;
    won_slots: number;
    canonical: number;
    orphaned: number;
    missed: number;
    future_rights: number;
    slot_start: number;
    slot_end: number;
    expected_rewards: number;
    earned_rewards: number;
    is_current: boolean;
  };
  sub_windows: {
    max: number;
    won_slots: number;
    canonical: number;
    orphaned: number;
    missed: number;
    future_rights: number;
    slot_start: number;
    slot_end: number;
    expected_rewards: number;
    earned_rewards: number;
    is_current: boolean;
  }[];
}

interface AllStatsResponse {
  won_slots: number;
  canonical: number;
  orphaned: number;
  missed: number;
  future_rights: number;
  expected_rewards: string;
  earned_rewards: string;
}

interface SlotResponse {
  slot: number;
  global_slot: number;
  block_status: BlockStatus;
  timestamp: number;
  state_hash: string | null;
  height: number | null;
  is_current_slot: boolean;
}

enum BlockStatus {
  Empty = 'Empty',
  ToBeProduced = 'ToBeProduced',
  Orphaned = 'Orphaned',
  OrphanedPending = 'OrphanedPending',
  Canonical = 'Canonical',
  CanonicalPending = 'CanonicalPending',
  Foreign = 'Foreign',
  Missed = 'Missed',
}
