import { Injectable } from '@angular/core';
import { Observable, of } from 'rxjs';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { CONFIG } from '@shared/constants/config';

@Injectable({
  providedIn: 'root',
})
export class AppService {

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
}
