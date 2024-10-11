import { Injectable } from '@angular/core';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { HttpClient } from '@angular/common/http';
import { EMPTY, map, Observable, of } from 'rxjs';
import { WebNodeService } from '@core/services/web-node.service';

@Injectable({
  providedIn: 'root',
})
export class RustService {

  private node: MinaNode;

  constructor(private http: HttpClient,
              private webNodeService: WebNodeService) {}

  changeRustNode(node: MinaNode): void {
    this.node = node;
  }

  get URL(): string {
    return this.node.url;
  }

  get name(): string {
    return this.node.name;
  }

  get<T>(path: string): Observable<T> {
    if (this.node.isWebNode) {
      return this.getFromWebNode(path).pipe(map((response: any) => {
        // console.log(path, response);
        return response;
      }));
    }
    return this.http.get<T>(this.URL + path);
  }

  post<T, B = string | object>(path: string, body: B): Observable<T> {
    if (this.node.isWebNode) {
      return this.postToWebNode(path, body).pipe(map((response: any) => {
        // console.log(path, response);
        return response;
      }));
    }
    return this.http.post<T>(this.URL + path, body);
  }

  getMemProfiler<T>(path: string): Observable<T> {
    return this.http.get<T>(this.node.memoryProfiler + path);
  }

  private getFromWebNode<T>(path: string): Observable<T> {
    switch (path) {
      case '/status':
        return this.webNodeService.status$;
      case '/state/peers':
        return this.webNodeService.peers$;
      case '/state/message-progress':
        return this.webNodeService.messageProgress$;
      case '/stats/sync?limit=1':
        return this.webNodeService.sync$;
      case '/stats/block_producer':
        return this.webNodeService.blockProducerStats$;
      case '/transaction-pool':
        return this.webNodeService.transactionPool$;
      case '/accounts':
        return this.webNodeService.accounts$;
      case '/best-chain-user-commands':
        return this.webNodeService.bestChainUserCommands$;
      default:
        throw new Error(`Web node doesn't support "${path}" path!`);
    }
  }

  private postToWebNode<T, B>(path: string, body: B): Observable<T> {
    switch (path) {
      case '/send-payment':
        return this.webNodeService.sendPayment$(body);
      default:
        throw new Error(`Web node doesn't support "${path}" path!`);
    }
  }
}
