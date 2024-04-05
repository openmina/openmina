import { Injectable } from '@angular/core';
import { BlockProductionModule } from '@app/features/block-production/block-production.module';
import { Observable, of } from 'rxjs';

export interface BlockProductionSlot {
  height: number;
  time: number;
  finished: boolean;
  canonical: boolean;
  orphaned: boolean;
  missed: boolean;
  missedRights: boolean;
  futureRights: boolean;
  active: boolean;
}

@Injectable({
  providedIn: BlockProductionModule,
})
export class BlockProductionOverviewService {

  constructor() { }

  getSlots(): Observable<BlockProductionSlot[]> {
    return of(this.generateRandomSlots());
  }

  private generateRandomSlots(): BlockProductionSlot[] {
    // generate slots interval  6732–7300
    // with random values
    // height is the index starting from 6732
    const slots = [];
    for (let i = 6732; i < 11000; i++) {
      slots.push({
        height: i,
        time: Math.floor(Math.random() * 100),
        finished: true,
        canonical: Math.random() > 0.95,
        orphaned: Math.random() > 0.95,
        missed: Math.random() > 0.95,
        missedRights: Math.random() > 0.99,
        futureRights: false,
        active: i === 10999,
      });
    }
    // rest push only slots where only futureRights can be true
    for (let i = 10000; i < 14010; i++) {
      slots.push({
        height: i,
        time: Math.floor(Math.random() * 100),
        finished: false,
        canonical: false,
        orphaned: false,
        missed: false,
        missedRights: false,
        futureRights: Math.random() > 0.98,
        active: false,
      });
    }
    return slots;
  }
}
