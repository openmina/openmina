import { Injectable } from '@angular/core';
import { map, Observable } from 'rxjs';
import { RustService } from '@core/services/rust.service';
import {
  NetworkBootstrapPeerType,
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';
import { ONE_MILLION } from '@openmina/shared';

@Injectable({
  providedIn: 'root',
})
export class NetworkBootstrapStatsService {

  constructor(private rust: RustService) { }

  getDhtBootstrapStats(): Observable<NetworkBootstrapStatsRequest[]> {
    return this.rust.get<BootstrapStatsResponse>('/discovery/bootstrap_stats').pipe(
      map((response: BootstrapStatsResponse) => this.mapBootstrapStats(response)),
    );
  }

  private mapBootstrapStats(response: BootstrapStatsResponse): NetworkBootstrapStatsRequest[] {
    return response.requests.map((request: BootstrapStatsRequest) => ({
      type: request.type,
      address: request.address,
      start: request.start,
      finish: request.finish,
      duration: request.finish ? (request.finish - request.start) / ONE_MILLION : undefined,
      peerId: request.peer_id,
      error: request.error,
      existingPeers: request.closest_peers?.filter(([, type]: [string, NetworkBootstrapPeerType]) => type === NetworkBootstrapPeerType.EXISTING).length || 0,
      newPeers: request.closest_peers?.filter(([, type]: [string, NetworkBootstrapPeerType]) => type === NetworkBootstrapPeerType.NEW).length || 0,
      closestPeers: request.closest_peers || [],
    }));
  }
}

interface BootstrapStatsResponse {
  requests: BootstrapStatsRequest[];
}

interface BootstrapStatsRequest {
  type: string;
  address: string;
  start: number;
  finish: number;
  peer_id: string;
  error: string | undefined;
  closest_peers: [string, NetworkBootstrapPeerType][];
}
