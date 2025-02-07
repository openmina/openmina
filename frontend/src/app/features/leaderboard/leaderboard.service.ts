import { Injectable } from '@angular/core';
import { combineLatest, map, Observable } from 'rxjs';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { collection, collectionData, CollectionReference, Firestore } from '@angular/fire/firestore';
import { WebNodeService } from '@core/services/web-node.service';
import { getElapsedTimeInMinsAndHours } from '@shared/helpers/date.helper';

@Injectable({
  providedIn: 'root',
})
export class LeaderboardService {

  private scoresCollection: CollectionReference;
  private maxScoreCollection: CollectionReference;

  constructor(private firestore: Firestore,
              private webnodeService: WebNodeService) {
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

        const items = scores.map(score => {
          return ({
            publicKey: score['publicKey'],
            blocksProduced: score['blocksProduced'],
            isActive: score['lastUpdated'] > Date.now() - 120000,
            uptimePercentage: this.getUptimePercentage(score['score'], maxScoreRightNow),
            uptimePrize: false,
            blocksPrize: false,
            score: score['score'],
            maxScore: maxScoreRightNow,
          } as HeartbeatSummary);
        });

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

  getUptime(): Observable<any> {
    const publicKey = this.webnodeService.privateStake.publicKey.replace('\n', '');

    return combineLatest([
      collectionData(this.scoresCollection, { idField: 'id' }),
      collectionData(this.maxScoreCollection, { idField: 'id' }),
    ]).pipe(
      map(([scores, maxScore]) => {
        const activeEntry = scores.find(score => score['publicKey'] === publicKey);

        return {
          uptimePercentage: this.getUptimePercentage(activeEntry['score'], maxScore[0]['value']),
          uptimeTime: getElapsedTimeInMinsAndHours(activeEntry['score'] * 5),
        };
      }),
    );
  }

  private getUptimePercentage(score: number, maxScore: number): number {
    let uptimePercentage = Number(((score / maxScore) * 100).toFixed(2));
    if (maxScore === 0) {
      uptimePercentage = 0;
    }
    return uptimePercentage;
  }
}
