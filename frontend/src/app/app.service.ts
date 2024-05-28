import { Injectable } from '@angular/core';
import { map, Observable, of } from 'rxjs';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { CONFIG } from '@shared/constants/config';
import { RustService } from '@core/services/rust.service';
import { AppNodeDetails } from '@shared/types/app/app-node-details.type';

@Injectable({
  providedIn: 'root',
})
export class AppService {

  constructor(private rust: RustService) { }

  getActiveNode(nodes: MinaNode[]): Observable<MinaNode> {
    const nodeName = new URL(location.href).searchParams.get('node');
    const configs = nodes;
    const nodeFromURL = configs.find(c => c.name === nodeName) || configs[0];
    return of(nodeFromURL);
  }

  getNodes(): Observable<MinaNode[]> {
    return of([
      ...CONFIG.configs,
      ...(localStorage.getItem('custom_nodes') ? JSON.parse(localStorage.getItem('custom_nodes')) : []),
    ]);
  }

  getActiveNodeDetails(): Observable<AppNodeDetails> {
    return of({
      status: 'Synced',
      blockHeight: 62453,
      blockTime: Date.now() - 50 * 1000,
      peers: 6,
      download: 1.1,
      upload: 0.5,
      transactions: 38,
      snarks: 4214,
    } as AppNodeDetails)
      // return this.rust.get<any>('/node-details')
      .pipe(
        map(data => ({ ...data })),
      );
  }
}
