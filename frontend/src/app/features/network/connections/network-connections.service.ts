import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { map, Observable } from 'rxjs';
import { ONE_MILLION, ONE_THOUSAND, toReadableDate } from '@openmina/shared';
import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';
import { NetworkConnection } from '@shared/types/network/connections/network-connection.type';
import { ConfigService } from '@core/services/config.service';

@Injectable({
  providedIn: 'root',
})
export class NetworkConnectionsService {

  constructor(private http: HttpClient,
              private config: ConfigService) { }

  getConnections(limit: number, id: number, direction: NetworkMessagesDirection): Observable<NetworkConnection[]> {
    let url = `${this.config.DEBUGGER}/connections?limit=${limit}&direction=${direction}`;

    if (id) {
      url += `&id=${id}`;
    }

    return this.http.get<any[]>(url)
      .pipe(map((messages: any[]) => this.mapNetworkConnectionsResponse(messages, direction)));
  }

  private mapNetworkConnectionsResponse(connections: any[], direction: NetworkMessagesDirection): NetworkConnection[] {
    if (direction === NetworkMessagesDirection.REVERSE) {
      connections = connections.reverse();
    }

    return connections.map(item => {
      const timestamp = (item[1].timestamp.secs_since_epoch * ONE_THOUSAND) + item[1].timestamp.nanos_since_epoch / ONE_MILLION;
      return ({
        connectionId: item[0],
        date: toReadableDate(timestamp),
        timestamp,
        incoming: item[1].incoming ? 'Incoming' : 'Outgoing',
        addr: item[1].info.addr,
        pid: item[1].info.pid,
        fd: item[1].info.fd,
        alias: item[1].alias,
        stats_in: item[1].stats_in,
        stats_out: item[1].stats_out,
      } as NetworkConnection);
    });
  }
}
