import { Injectable } from '@angular/core';
import { combineLatest, map, Observable } from 'rxjs';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { collection, collectionData, CollectionReference, Firestore } from '@angular/fire/firestore';

@Injectable({
  providedIn: 'root',
})
export class LeaderboardService {

  private scoresCollection: CollectionReference;
  private maxScoreCollection: CollectionReference;

  constructor(private firestore: Firestore) {
    this.scoresCollection = collection(this.firestore, 'scores');
    this.maxScoreCollection = collection(this.firestore, 'maxScore');
  }

  getHeartbeatsSummaries(): Observable<HeartbeatSummary[]> {
    return combineLatest([
      collectionData(this.scoresCollection, { idField: 'id' }),
      collectionData(this.maxScoreCollection, { idField: 'id' }),
    ]).pipe(
      map(([scores, maxScore]) => {
        const maxScoreRightNow = maxScore.find(c => c.id === 'current')['value'];

        const items = scores.map(score => ({
          publicKey: score['publicKey'],
          blocksProduced: score['blocksProduced'],
          isActive: score['lastUpdated'] > Date.now() - 120000,
          uptimePercentage: Math.floor((score['score'] / maxScoreRightNow) * 100),
          uptimePrize: false,
          blocksPrize: false,
        } as HeartbeatSummary));

        const sortedItemsByUptime = [...items].sort((a, b) => b.uptimePercentage - a.uptimePercentage);
        const fifthPlacePercentageByUptime = sortedItemsByUptime[4]?.uptimePercentage ?? 0;
        const highestProducedBlocks = Math.max(...items.map(item => item.blocksProduced));
        return items.map(item => ({
          ...item,
          uptimePrize: item.uptimePercentage >= fifthPlacePercentageByUptime,
          blocksPrize: item.blocksProduced === highestProducedBlocks,
        }));
      }),
    );
  }
}
