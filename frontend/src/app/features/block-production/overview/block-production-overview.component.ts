import { ChangeDetectionStrategy, Component, ElementRef, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  BlockProductionOverviewService,
} from '@app/features/block-production/overview/block-production-overview.service';

@Component({
  selector: 'mina-block-production-overview',
  templateUrl: './block-production-overview.component.html',
  styleUrls: ['./block-production-overview.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BlockProductionOverviewComponent extends StoreDispatcher implements OnInit {

  constructor(protected el: ElementRef) {super();}

  ngOnInit(): void {

  }
}
