import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  selectSnarksWorkPoolActiveWorkPoolDetail,
  selectSnarksWorkPoolActiveWorkPoolSpecs
} from '@snarks/work-pool/snarks-work-pool.state';
import { WorkPoolSpecs } from '@shared/types/snarks/work-pool/work-pool-specs.type';
import { downloadJsonFromURL } from '@openmina/shared';
import { RustService } from '@core/services/rust.service';
import { WorkPoolDetail } from '@shared/types/snarks/work-pool/work-pool-detail.type';

@Component({
  selector: 'mina-snarks-work-pool-details-specs',
  templateUrl: './snarks-work-pool-details-specs.component.html',
  styleUrls: ['./snarks-work-pool-details-specs.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SnarksWorkPoolDetailsSpecsComponent extends StoreDispatcher implements OnInit {

  activeWorkPool: WorkPoolSpecs;

  private jobId: string;

  constructor(private rust: RustService) {super();}

  ngOnInit(): void {
    this.select(selectSnarksWorkPoolActiveWorkPoolSpecs, (wp: WorkPoolSpecs) => {
      this.activeWorkPool = { ...wp };
      this.detect();
    });
    this.select(selectSnarksWorkPoolActiveWorkPoolDetail, (detail: WorkPoolDetail) => {
      this.jobId = detail.id;
    });
  }

  downloadBin(): void {
    downloadJsonFromURL(this.rust.URL + '/snarker/job/spec?id=' + this.jobId, 'work-pool-specs.bin', () => null);
  }

}
