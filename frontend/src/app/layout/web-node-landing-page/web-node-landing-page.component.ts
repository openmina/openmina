import { ChangeDetectionStrategy, Component, EventEmitter, OnInit, Output } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppSelectors } from '@app/app.state';
import { filter } from 'rxjs';

@Component({
  selector: 'mina-web-node-landing-page',
  standalone: true,
  templateUrl: './web-node-landing-page.component.html',
  styleUrl: './web-node-landing-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WebNodeLandingPageComponent extends StoreDispatcher implements OnInit {

  @Output() goToNode: EventEmitter<void> = new EventEmitter<void>();
  @Output() stopRequests: EventEmitter<void> = new EventEmitter<void>();

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, () => {
      this.stopRequests.emit();
    }, filter(Boolean));
  }
}
