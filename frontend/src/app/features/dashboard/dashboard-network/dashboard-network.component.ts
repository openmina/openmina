import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { DashboardPeersStats } from '@shared/types/dashboard/dashboard-peers-stats.type';
import { selectDashboardPeersStats } from '@dashboard/dashboard.state';
import { skip } from 'rxjs';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-dashboard-network',
  templateUrl: './dashboard-network.component.html',
  styleUrls: ['./dashboard-network.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column' },
})
export class DashboardNetworkComponent extends StoreDispatcher implements OnInit {

  stats: DashboardPeersStats = {
    connected: 0,
    disconnected: 0,
    connecting: 0,
  };

  ngOnInit(): void {
    this.listenToPeersChanges();
  }

  private listenToPeersChanges(): void {
    this.select(selectDashboardPeersStats, (stats: DashboardPeersStats) => {
      this.stats = stats;
      this.detect();
    }, skip(1));
  }
}
