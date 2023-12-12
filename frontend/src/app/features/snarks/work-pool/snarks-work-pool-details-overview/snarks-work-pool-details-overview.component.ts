import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectSnarksWorkPoolActiveWorkPoolDetail } from '@snarks/work-pool/snarks-work-pool.state';
import { WorkPoolDetail } from '@shared/types/snarks/work-pool/work-pool-detail.type';

@Component({
  selector: 'mina-snarks-work-pool-details-overview',
  templateUrl: './snarks-work-pool-details-overview.component.html',
  styleUrls: ['./snarks-work-pool-details-overview.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SnarksWorkPoolDetailsOverviewComponent extends StoreDispatcher implements OnInit {

  activeWorkPool: WorkPoolDetail;

  ngOnInit(): void {
    this.select(selectSnarksWorkPoolActiveWorkPoolDetail, (detail: WorkPoolDetail) => {
      this.activeWorkPool = detail;
      this.detect();
    });
  }
}
