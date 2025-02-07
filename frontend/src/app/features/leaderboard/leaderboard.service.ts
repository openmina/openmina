import { Injectable, Optional } from '@angular/core';
import { combineLatest, map, Observable } from 'rxjs';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { collection, collectionData, CollectionReference, Firestore, getDocs } from '@angular/fire/firestore';
import { WebNodeService } from '@core/services/web-node.service';
import { getElapsedTimeInMinsAndHours } from '@shared/helpers/date.helper';

@Injectable({
  providedIn: 'root',
})
export class LeaderboardService {

  private scoresCollection: CollectionReference;
  private maxScoreCollection: CollectionReference;

  private maxScoreRightNow: number;

  constructor(@Optional() private firestore: Firestore,
              private webnodeService: WebNodeService) {
    if (this.firestore) {
      this.scoresCollection = collection(this.firestore, 'scores');
      this.maxScoreCollection = collection(this.firestore, 'maxScore');
    }
  }

  getHeartbeatsSummaries(): Observable<HeartbeatSummary[]> {
    return combineLatest([
      collectionData(this.scoresCollection, { idField: 'id' }),
      collectionData(this.maxScoreCollection, { idField: 'id' }),
    ]).pipe(
      map(([scores, maxScore]) => {
        this.maxScoreRightNow = maxScore.find(c => c.id === 'current')['value'];

        const items = scores.map(score => {
          return ({
            publicKey: score['publicKey'],
            blocksProduced: score['blocksProduced'],
            isActive: score['lastUpdated'] > Date.now() - 120000,
            uptimePercentage: this.getUptimePercentage(score['score'], this.maxScoreRightNow),
            uptimePrize: false,
            blocksPrize: false,
            score: score['score'],
            maxScore: this.maxScoreRightNow,
          } as HeartbeatSummary);
        });

        const sortedItemsByUptime = [...items].sort((a, b) => b.uptimePercentage - a.uptimePercentage);
        const fifthPlacePercentageByUptime = sortedItemsByUptime[4]?.uptimePercentage ?? 0;
        const highestProducedBlocks = Math.max(
          ...items
            .filter(item => item.score > 0.3333 * this.maxScoreRightNow)
            .map(item => item.blocksProduced),
        );
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

        if (!activeEntry) {
          return {
            uptimePercentage: 0,
            uptimeTime: '',
          };
        }

        return {
          uptimePercentage: this.getUptimePercentage(activeEntry['score'], maxScore[0]['value']),
          uptimeTime: getElapsedTimeInMinsAndHours(activeEntry['score'] * 5),
        };
      }),
    );
  }

  private camelCaseToTitle(camelCase: string): string {
    return camelCase
      .replace(/([A-Z])/g, ' $1')
      .replace(/^./, match => match.toUpperCase());
  }

  private getUptimePercentage(score: number, maxScore: number): number {
    let uptimePercentage = Number(((score / maxScore) * 100).toFixed(2));
    if (maxScore === 0) {
      uptimePercentage = 0;
    }
    return uptimePercentage;
  }

  async downloadUptimeLottery(): Promise<void> {
    const querySnapshot = await getDocs(this.scoresCollection);
    const scoresData: any[] = [];

    querySnapshot.forEach((doc) => {
      scoresData.push({ id: doc.id, ...doc.data() });
    });

    const csvRows = [];

    let filteredData = scoresData
      .map(row => ({
        publicKey: row.publicKey,
        score: row.score,
      }))
      .filter(row => row.score > 0.3333 * this.maxScoreRightNow);
    filteredData = [...filteredData].sort((a, b) => b.score - a.score);

    const headers = ['publicKey', 'score'].map(header => this.camelCaseToTitle(header));
    csvRows.push(headers.join(','));

    filteredData.forEach((row: any) => {
      const values = headers.map(header => {
        const key = header.charAt(0).toLowerCase() + header.slice(1); // Convert to corresponding key
        const escape = ('' + row[key.replace(' ', '')]).replace(/"/g, '\\"');
        return `"${escape}"`;
      });
      csvRows.push(values.join(','));
    });

    const csvString = csvRows.join('\n');
    const blob = new Blob([csvString], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);

    const link = document.createElement('a');
    link.href = url;
    link.download = `export_${new Date().toISOString()}.csv`;
    link.click();

    URL.revokeObjectURL(url);
  }

  async downloadHighestUptime(): Promise<void> {
    const querySnapshot = await getDocs(this.scoresCollection);
    const scoresData: any[] = [];

    querySnapshot.forEach((doc) => {
      scoresData.push({ id: doc.id, ...doc.data() });
    });

    const csvRows = [];

    let filteredData = scoresData
      .map(row => ({
        publicKey: row.publicKey,
        score: row.score,
      }))
      .filter(row => row.score > 0.3333 * this.maxScoreRightNow);
    filteredData = [...filteredData].sort((a, b) => b.score - a.score);

    const sortedItemsByUptime = [...filteredData].sort((a, b) => b.score - a.score);
    const fifthPlaceByUptime = sortedItemsByUptime[4]?.score ?? 0;
    filteredData = filteredData.filter(row => row.score >= fifthPlaceByUptime);

    // Convert camelCase headers to Title Case with spaces
    const headers = ['publicKey', 'score'].map(header => this.camelCaseToTitle(header));
    csvRows.push(headers.join(','));

    filteredData.forEach((row: any) => {
      const values = headers.map(header => {
        const key = header.charAt(0).toLowerCase() + header.slice(1); // Convert to corresponding key
        const escape = ('' + row[key.replace(' ', '')]).replace(/"/g, '\\"');
        return `"${escape}"`;
      });
      csvRows.push(values.join(','));
    });

    const csvString = csvRows.join('\n');
    const blob = new Blob([csvString], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);

    const link = document.createElement('a');
    link.href = url;
    link.download = `export_${new Date().toISOString()}.csv`;
    link.click();

    URL.revokeObjectURL(url);
  }

  async downloadMostProducedBlocks(): Promise<void> {
    const querySnapshot = await getDocs(this.scoresCollection);
    const scoresData: any[] = [];

    querySnapshot.forEach((doc) => {
      scoresData.push({ id: doc.id, ...doc.data() });
    });

    const csvRows = [];

    let filteredData = scoresData
      .filter(row => row.score > 0.3333 * this.maxScoreRightNow)
      .map(row => ({
        publicKey: row.publicKey,
        blocksProduced: row.blocksProduced,
      }));
    filteredData = [...filteredData].sort((a, b) => b.blocksProduced - a.blocksProduced);

    const highestProducedBlocks = Math.max(...filteredData.map(row => row.blocksProduced));
    filteredData = filteredData.filter(row => row.blocksProduced === highestProducedBlocks);

    const headers = ['publicKey', 'blocksProduced'].map(header => this.camelCaseToTitle(header));
    csvRows.push(headers.join(','));

    filteredData.forEach((row: any) => {
      const values = headers.map(header => {
        const key = header.charAt(0).toLowerCase() + header.slice(1); // Convert to corresponding key
        const escape = ('' + row[key.replace(' ', '')]).replace(/"/g, '\\"');
        return `"${escape}"`;
      });
      csvRows.push(values.join(','));
    });

    const csvString = csvRows.join('\n');
    const blob = new Blob([csvString], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);

    const link = document.createElement('a');
    link.href = url;
    link.download = `export_${new Date().toISOString()}.csv`;
    link.click();

    URL.revokeObjectURL(url);
  }
}
