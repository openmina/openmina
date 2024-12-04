import { ChangeDetectionStrategy, Component, EventEmitter, OnInit, Output } from '@angular/core';
import { NgOptimizedImage } from '@angular/common';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppSelectors } from '@app/app.state';
import { filter } from 'rxjs';
import { ParseFilesComponent } from '@app/layout/parse-files/parse-files.component';

@Component({
  selector: 'mina-web-node-landing-page',
  standalone: true,
  imports: [
    NgOptimizedImage,
    ParseFilesComponent,
  ],
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
