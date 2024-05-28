import { Injectable } from '@angular/core';
import { AppSelectors } from '@app/app.state';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { filter } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class ConfigService {

  private node: MinaNode;

  constructor(private store: Store<MinaState>) { this.listenToNodeChanging(); }

  private listenToNodeChanging(): void {
    this.store.select(AppSelectors.activeNode)
      .pipe(filter(Boolean))
      .subscribe((node: MinaNode) => this.node = node);
  }

  get DEBUGGER(): string {
    return this.node.debugger;
  }

}
