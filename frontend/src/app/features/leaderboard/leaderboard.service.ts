import { Injectable } from '@angular/core';
import { Observable, of } from 'rxjs';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';

@Injectable({
  providedIn: 'root',
})
export class LeaderboardService {

  constructor() { }

  getHeartbeatsSummaries(): Observable<HeartbeatSummary[]> {
    const mockData: HeartbeatSummary[] = [
      {
        publicKey: '0x7a23c98f21345dc98e23a5b1c98d23b1c98d23b1c',
        isActive: true,
        uptimePercentage: 99.8,
        blocksProduced: 1243,
      },
      {
        publicKey: '0x8b34d09e32456ed09f34b6c2d09f34b6c2d09f34',
        isActive: true,
        uptimePercentage: 98.2,
        blocksProduced: 982,
      },
      {
        publicKey: '0x9c45e10f43567fe10a45c7d3e10a45c7d3e10a45',
        isActive: false,
        uptimePercentage: 45.6,
        blocksProduced: 234,
      },
      {
        publicKey: '0x0d56f21g54678gf21b56d8e4f21b56d8e4f21b56',
        isActive: true,
        uptimePercentage: 56.9,
        blocksProduced: 876,
      },
      {
        publicKey: '0x1e67g32h65789hg32c67e9f5g32c67e9f5g32c67',
        isActive: true,
        uptimePercentage: 23.1,
        blocksProduced: 1102,
      },
      {
        publicKey: '0x7a23c98f21345dc98e23a5b1c98d23b1c98d23b1c',
        isActive: true,
        uptimePercentage: 99.8,
        blocksProduced: 1243,
      },
      {
        publicKey: '0x8b34d09e32456ed09f34b6c2d09f34b6c2d09f34',
        isActive: false,
        uptimePercentage: 98.2,
        blocksProduced: 982,
      },
      {
        publicKey: '0x9c45e10f43567fe10a45c7d3e10a45c7d3e10a45',
        isActive: false,
        uptimePercentage: 45.6,
        blocksProduced: 234,
      },
      {
        publicKey: '0x0d56f21g54678gf21b56d8e4f21b56d8e4f21b56',
        isActive: true,
        uptimePercentage: 56.9,
        blocksProduced: 876,
      },
      {
        publicKey: '0x1e67g32h65789hg32c67e9f5g32c67e9f5g32c67',
        isActive: false,
        uptimePercentage: 23.1,
        blocksProduced: 1102,
      },
      {
        publicKey: '0x7a23c98f21345dc98e23a5b1c98d23b1c98d23b1c',
        isActive: false,
        uptimePercentage: 99.8,
        blocksProduced: 1243,
      },
      {
        publicKey: '0x8b34d09e32456ed09f34b6c2d09f34b6c2d09f34',
        isActive: false,
        uptimePercentage: 98.2,
        blocksProduced: 982,
      },
      {
        publicKey: '0x9c45e10f43567fe10a45c7d3e10a45c7d3e10a45',
        isActive: false,
        uptimePercentage: 45.6,
        blocksProduced: 234,
      },
      {
        publicKey: '0x0d56f21g54678gf21b56d8e4f21b56d8e4f21b56',
        isActive: true,
        uptimePercentage: 56.9,
        blocksProduced: 876,
      },
      {
        publicKey: '0x1e67g32h65789hg32c67e9f5g32c67e9f5g32c67',
        isActive: true,
        uptimePercentage: 23.1,
        blocksProduced: 1102,
      },
      {
        publicKey: '0x7a23c98f21345dc98e23a5b1c98d23b1c98d23b1c',
        isActive: true,
        uptimePercentage: 99.8,
        blocksProduced: 1243,
      },
      {
        publicKey: '0x8b34d09e32456ed09f34b6c2d09f34b6c2d09f34',
        isActive: true,
        uptimePercentage: 98.2,
        blocksProduced: 982,
      },
      {
        publicKey: '0x9c45e10f43567fe10a45c7d3e10a45c7d3e10a45',
        isActive: false,
        uptimePercentage: 45.6,
        blocksProduced: 234,
      },
      {
        publicKey: '0x0d56f21g54678gf21b56d8e4f21b56d8e4f21b56',
        isActive: true,
        uptimePercentage: 56.9,
        blocksProduced: 876,
      },
      {
        publicKey: '0x1e67g32h65789hg32c67e9f5g32c67e9f5g32c67',
        isActive: true,
        uptimePercentage: 23.1,
        blocksProduced: 1102,
      },
      {
        publicKey: '0x7a23c98f21345dc98e23a5b1c98d23b1c98d23b1c',
        isActive: true,
        uptimePercentage: 99.8,
        blocksProduced: 1243,
      },
      {
        publicKey: '0x8b34d09e32456ed09f34b6c2d09f34b6c2d09f34',
        isActive: true,
        uptimePercentage: 98.2,
        blocksProduced: 982,
      },
      {
        publicKey: '0x9c45e10f43567fe10a45c7d3e10a45c7d3e10a45',
        isActive: false,
        uptimePercentage: 45.6,
        blocksProduced: 234,
      },
      {
        publicKey: '0x0d56f21g54678gf21b56d8e4f21b56d8e4f21b56',
        isActive: true,
        uptimePercentage: 56.9,
        blocksProduced: 876,
      },
      {
        publicKey: '0x1e67g32h65789hg32c67e9f5g32c67e9f5g32c67',
        isActive: true,
        uptimePercentage: 23.1,
        blocksProduced: 1102,
      },
      {
        publicKey: '0x7a23c98f21345dc98e23a5b1c98d23b1c98d23b1c',
        isActive: true,
        uptimePercentage: 99.8,
        blocksProduced: 1243,
      },
      {
        publicKey: '0x8b34d09e32456ed09f34b6c2d09f34b6c2d09f34',
        isActive: false,
        uptimePercentage: 98.2,
        blocksProduced: 982,
      },
      {
        publicKey: '0x9c45e10f43567fe10a45c7d3e10a45c7d3e10a45',
        isActive: false,
        uptimePercentage: 45.6,
        blocksProduced: 234,
      },
      {
        publicKey: '0x0d56f21g54678gf21b56d8e4f21b56d8e4f21b56',
        isActive: true,
        uptimePercentage: 56.9,
        blocksProduced: 876,
      },
      {
        publicKey: '0x1e67g32h65789hg32c67e9f5g32c67e9f5g32c67',
        isActive: false,
        uptimePercentage: 23.1,
        blocksProduced: 1102,
      },
      {
        publicKey: '0x7a23c98f21345dc98e23a5b1c98d23b1c98d23b1c',
        isActive: false,
        uptimePercentage: 99.8,
        blocksProduced: 1243,
      },
      {
        publicKey: '0x8b34d09e32456ed09f34b6c2d09f34b6c2d09f34',
        isActive: false,
        uptimePercentage: 98.2,
        blocksProduced: 982,
      },
      {
        publicKey: '0x9c45e10f43567fe10a45c7d3e10a45c7d3e10a45',
        isActive: false,
        uptimePercentage: 45.6,
        blocksProduced: 234,
      },
      {
        publicKey: '0x0d56f21g54678gf21b56d8e4f21b56d8e4f21b56',
        isActive: true,
        uptimePercentage: 56.9,
        blocksProduced: 876,
      },
      {
        publicKey: '0x1e67g32h65789hg32c67e9f5g32c67e9f5g32c67',
        isActive: true,
        uptimePercentage: 23.1,
        blocksProduced: 1102,
      },
      {
        publicKey: '0x7a23c98f21345dc98e23a5b1c98d23b1c98d23b1c',
        isActive: true,
        uptimePercentage: 99.8,
        blocksProduced: 1243,
      },
      {
        publicKey: '0x8b34d09e32456ed09f34b6c2d09f34b6c2d09f34',
        isActive: true,
        uptimePercentage: 98.2,
        blocksProduced: 982,
      },
      {
        publicKey: '0x9c45e10f43567fe10a45c7d3e10a45c7d3e10a45',
        isActive: false,
        uptimePercentage: 45.6,
        blocksProduced: 234,
      },
      {
        publicKey: '0x0d56f21g54678gf21b56d8e4f21b56d8e4f21b56',
        isActive: true,
        uptimePercentage: 56.9,
        blocksProduced: 876,
      },
      {
        publicKey: '0x1e67g32h65789hg32c67e9f5g32c67e9f5g32c67',
        isActive: true,
        uptimePercentage: 23.1,
        blocksProduced: 1102,
      },
    ];

    return of(mockData);
  }
}
