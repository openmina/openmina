import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { NetworkConnectionsClose, NetworkConnectionsInit } from '@network/connections/network-connections.actions';
import { selectNetworkConnectionsActiveConnection } from '@network/connections/network-connections.state';
import { NetworkConnection } from '@shared/types/network/connections/network-connection.type';
import { selectActiveNode } from '@app/app.state';
import { filter } from 'rxjs';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-network-connections',
  templateUrl: './network-connections.component.html',
  styleUrls: ['./network-connections.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100' },
})
export class NetworkConnectionsComponent extends StoreDispatcher implements OnInit, OnDestroy {

  isActiveRow: boolean = false;

  constructor(public el: ElementRef) { super(); }

  ngOnInit(): void {
    this.listenToActiveRowChange();
    this.listenToActiveNodeChange();
  }

  private listenToActiveNodeChange(): void {
    this.select(selectActiveNode, () => {
      this.dispatch(NetworkConnectionsInit);
    }, filter(Boolean));
  }

  private listenToActiveRowChange(): void {
    this.select(selectNetworkConnectionsActiveConnection, (row: NetworkConnection) => {
      if (row && !this.isActiveRow) {
        this.isActiveRow = true;
        this.detect();
      } else if (!row && this.isActiveRow) {
        this.isActiveRow = false;
        this.detect();
      }
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(NetworkConnectionsClose);
  }
}
