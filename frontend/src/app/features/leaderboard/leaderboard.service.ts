import { Injectable, Optional } from '@angular/core';
import { combineLatest, map, Observable } from 'rxjs';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { collection, collectionData, CollectionReference, Firestore, getDocs } from '@angular/fire/firestore';
import { WebNodeService } from '@core/services/web-node.service';
import { getElapsedTimeInMinsAndHours } from '@shared/helpers/date.helper';
import { ONE_THOUSAND, toReadableDate } from '@openmina/shared';

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
        // this.printHeartbeats(scores);
        const maxScoreNow: any = maxScore.find(c => c.id === 'current');
        this.maxScoreRightNow = maxScoreNow ? maxScoreNow['value'] : 0;
        const items = scores.map(score => {
          const isWhale = score['publicKey'].includes('B62qkiqPXFDayJV8JutYvjerERZ35EKrdmdcXh3j1rDUHRs1bJkFFcX') || score['publicKey'].includes('B62qpQT46XiGQs7KhcczifvvYcnx7fbTzKj8a83UcT2BhPEs5mYnzdp');
          return ({
            publicKey: score['publicKey'],
            blocksProduced: score['blocksProduced'],
            isWhale,
            uptimePercentage: this.getUptimePercentage(score['score'], this.maxScoreRightNow),
            uptimePrize: false,
            blocksPrize: false,
            score: score['score'],
            maxScore: this.maxScoreRightNow,
          } as HeartbeatSummary);
        });

        const sortedItemsByUptime = [...items].filter(i => !i.isWhale).sort((a, b) => b.uptimePercentage - a.uptimePercentage);
        const fifthPlacePercentageByUptime = sortedItemsByUptime[4]?.uptimePercentage ?? 0;
        const highestProducedBlocks = Math.max(
          ...items
            .filter(i => !i.isWhale)
            .filter(item => item.score > 0.3333 * this.maxScoreRightNow)
            .map(item => item.blocksProduced),
        );
        return items.map(item => ({
          ...item,
          uptimePrize: item.isWhale ? false : (item.uptimePercentage >= fifthPlacePercentageByUptime),
          blocksPrize: item.isWhale ? false : (item.blocksProduced === highestProducedBlocks),
        }));
      }),
    );
  }

  one: any;

  printHeartbeats(heartbeats: any[]): void {
    if (this.one) {
      return;
    }
    this.one = 1;
    // Sort the heartbeats by createTime (oldest first)
    const sortedHeartbeats = [...heartbeats].sort((a, b) => {
      const timeA = a.createTime.seconds * 1000 + a.createTime.nanoseconds / 1000000;
      const timeB = b.createTime.seconds * 1000 + b.createTime.nanoseconds / 1000000;
      return timeA - timeB;
    });

    // Create an array of {time, publicKey} objects
    const formattedData = sortedHeartbeats.map(heartbeat => {
      // Convert seconds and nanoseconds to milliseconds for Date constructor
      const milliseconds = heartbeat.createTime.seconds * 1000 + heartbeat.createTime.nanoseconds / 1000000;
      const date = new Date(milliseconds);

      // Get full day name
      const dayNames = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
      const fullDayName = dayNames[date.getUTCDay()];

      // Format in UTC with full day name
      const utcString = date.toUTCString();
      const formattedTime = utcString.replace(/^[A-Za-z]{3},/, `${fullDayName},`);

      return {
        time: formattedTime,
        'Public Key': heartbeat.submitter
      };
    });

    const csvRows = [];

    // Define headers to match the property names exactly
    const headers = ['Public Key', 'time'];
    csvRows.push(headers.join(','));

    // Map rows by accessing properties directly
    formattedData.forEach((row) => {
      // Make sure to escape any commas within the values
      const publicKey = `"${row['Public Key']}"`;
      const time = `"${row['time']}"`;

      // Join the values with a comma to create a CSV row
      csvRows.push(`${publicKey},${time}`);
    });

    const csvString = csvRows.join('\n');
    const blob = new Blob([csvString], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);

    const link = document.createElement('a');
    link.href = url;
    link.download = `All heartbeats ${new Date().toISOString().replace(/:/g, '-')}.csv`;
    link.click();

    URL.revokeObjectURL(url);
  }

  getUptime(): Observable<{ uptimePercentage: number, uptimeTime: string }> {
    const publicKey = this.webnodeService.privateStake?.publicKey?.replace('\n', '');

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

  async downloadAll(): Promise<void> {
    const querySnapshot = await getDocs(this.scoresCollection);
    const scoresData: any[] = [];

    querySnapshot.forEach((doc) => {
      scoresData.push({ id: doc.id, ...doc.data() });
    });

    const csvRows = [];

    let filteredData = scoresData
      .map(row => ({
        publicKey: row.publicKey,
        score: row.score + ' / ' + this.maxScoreRightNow,
        uptime: this.getUptimePercentage(row.score, this.maxScoreRightNow) + '%',
        uptimeTime: row.score,
        producedBlocks: row.blocksProduced,
        lastUpdated: toReadableDate(row.lastUpdated * ONE_THOUSAND),
      }));
    filteredData = [...filteredData].sort((a, b) => b.uptimeTime - a.uptimeTime);

    const headers = ['publicKey', 'score', 'uptime', /*'lastUpdated',*/ 'producedBlocks'].map(header => this.camelCaseToTitle(header));
    csvRows.push(headers.join(','));

    // Map rows
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
    link.download = `export_${new Date().toISOString().replace(/:/g, '-')}.csv`;
    link.click();

    URL.revokeObjectURL(url);
  }
}
