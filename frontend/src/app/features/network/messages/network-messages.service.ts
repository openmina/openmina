import { Injectable } from '@angular/core';
import { HttpClient, HttpEventType, HttpRequest } from '@angular/common/http';
import { catchError, map, Observable, of } from 'rxjs';
import { ONE_MILLION, ONE_THOUSAND, toReadableDate } from '@openmina/shared';
import { NetworkMessage } from '@shared/types/network/messages/network-message.type';
import { NetworkMessageConnection } from '@shared/types/network/messages/network-messages-connection.type';
import { NetworkMessagesFilter } from '@shared/types/network/messages/network-messages-filter.type';
import { NetworkMessagesFilterTypes } from '@shared/types/network/messages/network-messages-filter-types.enum';
import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';
import { ConfigService } from '@core/services/config.service';


@Injectable({
  providedIn: 'root',
})
export class NetworkMessagesService {

  constructor(private http: HttpClient,
              private config: ConfigService) { }

  getNetworkMessages(limit: number, id: number | undefined, direction: NetworkMessagesDirection, activeFilters: NetworkMessagesFilter[], from: number | undefined, to: number | undefined): Observable<NetworkMessage[]> {
    let url = `${this.config.DEBUGGER}/messages?limit=${limit}&direction=${direction}`;

    if (id) {
      url += `&id=${id}`;
    }
    if (from) {
      url += `&timestamp=${from}`;
    }
    if (to) {
      url += `&timestamp_limit=${to}`;
    }
    Object.values(NetworkMessagesFilterTypes).forEach((filterType: string) => {
      if (activeFilters.some(f => f.type === filterType)) {
        url += `&${filterType}=` + activeFilters.filter(f => f.type === filterType).map(f => f.value).join(',');
      }
    });

    return this.http.get<any[]>(url)
      .pipe(map((messages: any[]) => this.mapNetworkMessageResponse(messages, direction)));
  }

  getNetworkFullMessage(messageId: number): Observable<any> {
    return this.http.request<any>(new HttpRequest<any>('GET', `${this.config.DEBUGGER}/message/` + messageId, { reportProgress: true, observe: 'events' }))
      .pipe(
        map((event: any) => {
          if (event.type === HttpEventType.DownloadProgress && event.total > 10485760) {
            throw new Error(event.total);
          } else if (event.type === HttpEventType.Response) {
            return event.body.message;
          }
        }),
        catchError((err: Error) => of(err.message)),
      );
  }

  getNetworkConnection(connectionId: number): Observable<NetworkMessageConnection> {
    return this.http.get<any>(`${this.config.DEBUGGER}/connection/` + connectionId)
      .pipe(map(NetworkMessagesService.mapNetworkConnectionResponse));
  }

  getNetworkMessageHex(messageId: number): Observable<string> {
    return this.http.get<string>(`${this.config.DEBUGGER}/message_hex/` + messageId);
  }

  private mapNetworkMessageResponse(messages: any[], direction: NetworkMessagesDirection): NetworkMessage[] {
    if (direction === NetworkMessagesDirection.REVERSE) {
      messages = messages.reverse();
    }

    return messages.map(message => ({
      id: message[0],
      timestamp: toReadableDate((message[1].timestamp.secs_since_epoch * ONE_THOUSAND) + message[1].timestamp.nanos_since_epoch / ONE_MILLION),
      incoming: message[1].incoming ? 'Incoming' : 'Outgoing',
      connectionId: message[1].connection_id,
      address: message[1].remote_addr,
      size: message[1].size,
      streamKind: message[1].stream_kind,
      failedToDecryptPercentage: message[1].message?.total_failed ? NetworkMessagesService.getFailedToDecryptPercentage(message) : undefined,
      messageKind: NetworkMessagesService.getMessageKind(message),
    } as NetworkMessage));
  }

  private static getFailedToDecryptPercentage(message: any): number {
    return Number((100 * message[1].message.total_failed / (message[1].message.total_decrypted + message[1].message.total_failed)).toFixed(1));
  }

  private static getMessageKind(message: any): string {
    return message[1].message;
    // const messageKind = message[1].message[0]?.message?.type
    //   ?? message[1].message[0]?.type
    //   ?? message[1].message.type
    //   ?? message[1].message.action;
    // if (messageKind) {
    //   return messageKind;
    // }
    // return typeof message[1].message === 'string' ? message[1].message : 'Error Report';
  }

  private static mapNetworkConnectionResponse(connection: any): NetworkMessageConnection {
    return {
      address: connection.info.addr,
      pid: connection.info.pid,
      fd: connection.info.fd,
      incoming: connection.incoming,
      timestamp: toReadableDate(connection.timestamp.secs_since_epoch * ONE_THOUSAND),
    } as NetworkMessageConnection;
  }
}
