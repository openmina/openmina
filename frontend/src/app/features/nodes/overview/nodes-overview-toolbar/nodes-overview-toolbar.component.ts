import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-nodes-overview-toolbar',
  templateUrl: './nodes-overview-toolbar.component.html',
  styleUrls: ['./nodes-overview-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row h-xl' },
})
export class NodesOverviewToolbarComponent extends StoreDispatcher implements OnInit {

  ngOnInit(): void {
  }

}
