import { Injectable } from '@angular/core';
import { MinaNode, MinaNodeType } from '@shared/types/core/environment/mina-env.type';
import { HttpClient } from '@angular/common/http';
import { EMPTY, map, Observable, of } from 'rxjs';
import { WebNodeService } from '@core/services/web-node.service';
import { GraphQLService } from '@core/services/graph-ql.service';

@Injectable({
  providedIn: 'root',
})
export class ApiService {
  private node: MinaNode;

  constructor(private http: HttpClient,
              private graphQlService: GraphQLService,
              private webNodeService: WebNodeService) {}

  changeRustNode(node: MinaNode): void {
    this.node = node;
    this.graphQlService.changeGraphQlProvider(node);
  }

  get activeNodeIsWebNode(): boolean {
    return this.node.isWebNode;
  }

  get URL(): string {
    return this.node.url;
  }

  get name(): string {
    return this.node.name;
  }

  get<T>(path: string, data?: any): Observable<T> {
    if (this.node.isWebNode) {
      return this.getFromWebNode(path);
    } else if (this.node.type === MinaNodeType.RUST || !this.node.type) {
      return this.http.get<T>(this.URL + path);
    } else if (this.node.type === MinaNodeType.OCAML) {
      return this.graphQlService.get<T>(path, data);
    } else {
      throw new Error(`Unknown node type: ${this.node.type}`);
    }
  }

  post<T, B = string | object>(path: string, body: B): Observable<T> {
    if (this.node.isWebNode) {
      return this.postToWebNode(path, body);
    } else if (this.node.type === MinaNodeType.RUST || !this.node.type) {
      return this.http.post<T>(this.URL + path, body);
    } else if (this.node.type === MinaNodeType.OCAML) {
      return this.graphQlService.post<T>(path, body);
    } else {
      throw new Error(`Unknown node type: ${this.node.type}`);
    }
  }

  getMemProfiler<T>(path: string): Observable<T> {
    return this.http.get<T>(this.node.memoryProfiler + path);
  }

  private getFromWebNode<T>(path: string): Observable<T> {
    if (path.includes('/stats/actions')) {
      return this.webNodeService.actions$(path);
    }
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
      case '/build_env':
        return this.webNodeService.envBuildDetails$;
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
