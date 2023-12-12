import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectDashboardPeersStats } from '@dashboard/dashboard.state';
import { DashboardPeersStats } from '@shared/types/dashboard/dashboard-peers-stats.type';
import { skip } from 'rxjs';

@Component({
  selector: 'mina-dashboard-peers',
  templateUrl: './dashboard-peers.component.html',
  styleUrls: ['./dashboard-peers.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column' },
})
export class DashboardPeersComponent extends StoreDispatcher implements OnInit {

  stats: DashboardPeersStats;

  ngOnInit(): void {
    this.listenToPeersChanges();
  }

  private listenToPeersChanges(): void {
    this.select(selectDashboardPeersStats, (stats: DashboardPeersStats) => {
      this.stats = stats;
    }, skip(1));
  }
}
