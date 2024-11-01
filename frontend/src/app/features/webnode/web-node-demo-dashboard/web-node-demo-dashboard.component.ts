import { ChangeDetectionStrategy, Component, NgZone, OnInit } from '@angular/core';
import { untilDestroyed } from '@ngneat/until-destroy';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { WebNodeService } from '@core/services/web-node.service';
import { GlobalErrorHandlerService } from '@openmina/shared';
import { NgClass, NgForOf, NgIf } from '@angular/common';
import { Router } from '@angular/router';
import { getFirstFeature } from '@shared/constants/config';
import { trigger, state, style, transition, animate } from '@angular/animations';
import { switchMap, timer } from 'rxjs';

export interface WebNodeDemoLoadingStep {
  name: string;
  loaded: boolean;
  attempt?: number;
  step: number;
}


@Component({
  selector: 'mina-web-node-demo-dashboard',
  templateUrl: './web-node-demo-dashboard.component.html',
  styleUrls: ['./web-node-demo-dashboard.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100 w-100 align-center' },
  standalone: true,
  imports: [
    NgClass,
    NgIf,
    NgForOf,
  ],
  animations: [
    trigger('fadeIn', [
      state('void', style({ opacity: 0 })),
      state('*', style({ opacity: 1 })),
      transition('void => *', [
        animate('.6s ease-in'),
      ]),
    ]),
  ],
})
export class WebNodeDemoDashboardComponent extends StoreDispatcher implements OnInit {

  readonly loading: WebNodeDemoLoadingStep[] = [
    { name: 'Setting up browser for Web Node', loaded: false, step: 1 },
    { name: 'Getting ready to produce blocks', loaded: false, step: 2 },
    { name: 'Connecting directly to Mina network', loaded: false, step: 3 },
  ];
  ready: boolean = false;
  errors: string[] = [];

  constructor(private errorHandler: GlobalErrorHandlerService,
              private webNodeService: WebNodeService,
              private zone: NgZone,
              private router: Router) { super(); }

  ngOnInit(): void {
    this.listenToErrorIssuing();
    this.checkWebNodeProgress();
    this.fetchPeersInformation();
  }

  private checkWebNodeProgress(): void {
    this.webNodeService.webnodeProgress$.pipe(untilDestroyed(this)).subscribe((state: string) => {
      if (state === 'Loaded') {
        this.loading[0].loaded = true;
      } else if (state === 'Started') {
        this.loading[0].loaded = true;
        this.loading[1].loaded = true;
      } else if (state === 'Connected') {
        this.loading.forEach((step: WebNodeDemoLoadingStep) => step.loaded = true);
      }
      this.ready = this.loading.every((step: WebNodeDemoLoadingStep) => step.loaded);
      this.detect();
    });
  }

  private fetchPeersInformation(): void {
    timer(0, 1000).pipe(
      switchMap(() => this.webNodeService.peers$),
      untilDestroyed(this),
    ).subscribe();
  }

  private listenToErrorIssuing(): void {
    this.errorHandler.errors$
      .pipe(untilDestroyed(this))
      .subscribe((error: string) => {
        console.log(error);
      });
  }

  goToDashboard(): void {
    if (!this.ready) {
      return;
    }
    this.router.navigate([getFirstFeature()]);
  }
}
