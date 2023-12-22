import { Injectable } from '@angular/core';
import { map, Observable } from 'rxjs';
import { DashboardPeer, DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { RustService } from '@core/services/rust.service';
import { ONE_MILLION, toReadableDate } from '@openmina/shared';

@Injectable({ providedIn: 'root' })
export class DashboardService {

  constructor(private rust: RustService) { }

  getPeers(): Observable<DashboardPeer[]> {
    return this.rust.get<PeersResponse[]>('/state/peers').pipe(
      map((response: PeersResponse[]) =>
        response.map((peer: PeersResponse) => ({
          peerId: peer.peer_id,
          bestTip: peer.best_tip,
          globalSlot: peer.best_tip_global_slot,
          bestTipDatetime: peer.best_tip_timestamp ? toReadableDate(peer.best_tip_timestamp / ONE_MILLION) : '-',
          bestTipTimestamp: peer.best_tip_timestamp,
          datetime: toReadableDate(peer.time / ONE_MILLION),
          timestamp: peer.time,
          height: peer.best_tip_height,
          status: peer.connection_status,
          address: peer.address,
        } as DashboardPeer)),
      ),
    );
  }
}

interface PeersResponse {
  peer_id: string;
  best_tip: string | null;
  time: number;
  best_tip_height: number;
  best_tip_global_slot: number;
  best_tip_timestamp: number;
  connection_status: DashboardPeerStatus;
  address: string | null;
}
