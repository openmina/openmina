import { ChangeDetectionStrategy, Component, ElementRef, NgZone, OnInit, ViewChild } from '@angular/core';
import { untilDestroyed } from '@ngneat/until-destroy';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { WebNodeService } from '@core/services/web-node.service';
import { GlobalErrorHandlerService } from '@openmina/shared';
import { NgClass, NgForOf, NgIf, NgOptimizedImage } from '@angular/common';
import { Router } from '@angular/router';
import { getFirstFeature } from '@shared/constants/config';
import { trigger, state, style, transition, animate } from '@angular/animations';
import { filter, switchMap, timer } from 'rxjs';
import { LoadingSpinnerComponent } from '@shared/loading-spinner/loading-spinner.component';
import { FileProgressHelper } from '@core/helpers/file-progress.helper';
import * as d3 from 'd3';

export enum WebNodeStepStatus {
  DONE,
  LOADING,
  PENDING,
}

export interface WebNodeLoadingStep {
  name: string;
  loaded: boolean;
  status: WebNodeStepStatus;
  data: {
    downloaded: string;
    total: string;
  } | any;
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
    NgOptimizedImage,
    LoadingSpinnerComponent,
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

  protected readonly WebNodeStepStatus = WebNodeStepStatus;
  readonly loading: WebNodeLoadingStep[] = [
    { name: 'Setting up browser for Web Node', loaded: false, status: WebNodeStepStatus.LOADING, data: null },
    { name: 'Getting ready to produce blocks', loaded: false, status: WebNodeStepStatus.PENDING, data: { est: '~5s' } },
    {
      name: 'Connecting directly to Mina network',
      loaded: false,
      status: WebNodeStepStatus.PENDING,
      data: { est: '~2s' },
    },
  ];
  ready: boolean = false;
  errors: string[] = [];

  private stepsPercentages = this.stepPercentages;
  private secondStepInterval: any;
  private thirdStepInterval: any;
  private progress: number = 0;
  private svg: any;
  private progressBar: any;
  private arc: any;
  @ViewChild('progress', { static: true }) private chartContainer: ElementRef<HTMLDivElement>;

  constructor(private errorHandler: GlobalErrorHandlerService,
              private webNodeService: WebNodeService,
              private router: Router) { super(); }

  ngOnInit(): void {
    this.fetchProgress();
    this.listenToErrorIssuing();
    this.checkWebNodeProgress();
    this.fetchPeersInformation();
    this.buildProgressBar();
  }

  private checkWebNodeProgress(): void {
    this.webNodeService.webnodeProgress$.pipe(untilDestroyed(this)).subscribe((state: string) => {
      if (state === 'Loaded') {
        this.loading[0].loaded = true;
        this.loading[0].status = WebNodeStepStatus.DONE;
        this.loading[1].status = WebNodeStepStatus.LOADING;
        this.advanceProgressFor2ndStep();
      } else if (state === 'Started') {
        clearInterval(this.secondStepInterval);
        this.loading[0].loaded = true;
        this.loading[1].loaded = true;
        this.loading[0].status = WebNodeStepStatus.DONE;
        this.loading[1].status = WebNodeStepStatus.DONE;
        this.loading[2].status = WebNodeStepStatus.LOADING;
        this.updateProgressBar(this.stepsPercentages[0] + this.stepsPercentages[1]);
        this.advanceProgressFor3rdStep();
      } else if (state === 'Connected') {
        clearInterval(this.thirdStepInterval);
        this.loading[0].status = WebNodeStepStatus.DONE;
        this.loading[1].status = WebNodeStepStatus.DONE;
        this.loading[2].status = WebNodeStepStatus.DONE;
        this.loading.forEach((step: WebNodeLoadingStep) => step.loaded = true);
        this.goToEndProgress();
      }
      this.ready = this.loading.every((step: WebNodeLoadingStep) => step.loaded);
      this.detect();
    });
  }

  private get stepPercentages(): number[] {
    // random between 65 and 80
    const first = Math.floor(Math.random() * 15) + 65;
    // random between 15 and 17
    const second = Math.floor(Math.random() * 2) + 15;
    const third = 100 - first - second;
    return [first, second, third];
  }

  private advanceProgressFor2ndStep(): void {
    // first step is done, now we are working on the second step.
    // each second add 1% to the progress.
    // but the second step is only 10% of the total progress.
    // so never go above 10%. Stop at 9% if the second step is not done yet.
    let progress = 0;
    this.secondStepInterval = setInterval(() => {
      if (progress < this.stepsPercentages[1] - 1) {
        progress += 0.125;
        this.updateProgressBar(this.stepsPercentages[0] + progress);
      }
    }, 75);
  }

  private advanceProgressFor3rdStep(): void {
    // second step is done, now we are working on the third step.
    // each second add 1% to the progress.
    // but the third step is only 10% of the total progress.
    // so never go above 10%. Stop at 9% if the third step is not done yet.
    let progress = 0;
    this.thirdStepInterval = setInterval(() => {
      if (progress < this.stepsPercentages[2] - 1) {
        progress += 0.125;
        this.updateProgressBar(this.stepsPercentages[0] + this.stepsPercentages[1] + progress);
      }
    }, 75);
  }

  private goToEndProgress(): void {
    // increase it to 100% to make sure it's done.
    // make it smooth, do a lot of increases from where it left
    // to make it look like it's finishing.

    let progress = this.progress;
    let interval = setInterval(() => {
      if (progress < 100) {
        progress += 1;
        this.updateProgressBar(progress);
      } else {
        clearInterval(interval);
      }
    }, 10);
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

  private fetchProgress(): void {
    FileProgressHelper.progress$.pipe(
      filter(Boolean),
      untilDestroyed(this),
    ).subscribe((progress) => {
      this.loading[0].data = {
        downloaded: (progress.downloaded / 1e6).toFixed(1),
        total: (progress.totalSize / 1e6).toFixed(1),
      };
      // this step is only 1 out of 3. But it counts as 80% of the 100% progress.
      // So we need to calculate the total progress based on the current step.
      const totalProgress = (progress.progress * this.stepsPercentages[0]) / 100;
      this.updateProgressBar(totalProgress);
      this.detect();
    });
  }

  goToDashboard(): void {
    if (!this.ready) {
      return;
    }
    this.router.navigate([getFirstFeature()]);
  }

  private buildProgressBar(): void {
    const width = this.chartContainer.nativeElement.offsetWidth;
    const height = this.chartContainer.nativeElement.offsetHeight;
    const barWidth = 12;
    const progress = 0;

    this.svg = d3
      .select(this.chartContainer.nativeElement)
      .append('svg')
      .attr('width', width)
      .attr('height', height);

    this.progressBar = this.svg.append('g')
      .attr('transform', `translate(${width / 2}, ${height / 2})`);

    const radius = Math.min(width, height) / 2 - barWidth;
    this.arc = d3.arc()
      .innerRadius(radius - barWidth)
      .outerRadius(radius)
      .startAngle(0)
      .endAngle(Math.PI * 2 * (progress / 100));

    this.progressBar.append('path')
      .attr('d', this.arc)
      .attr('opacity', 0.8)
      .attr('fill', 'url(#progress-gradient)');

    const defs = this.svg.append('defs');
    const gradient = defs.append('linearGradient')
      .attr('id', 'progress-gradient')
      .attr('x1', '0%')
      .attr('x2', '100%')
      .attr('y1', '0%')
      .attr('y2', '0%')
      .attr('gradientTransform', 'rotate(45)');

    gradient.append('stop')
      .attr('offset', '8%')
      .attr('stop-color', '#57D7FF');
    gradient.append('stop')
      .attr('offset', '60%')
      .attr('stop-color', '#FDA2FF');
    gradient.append('stop')
      .attr('offset', '100%')
      .attr('stop-color', '#FF833D');

    this.progressBar.append('text')
      .attr('text-anchor', 'middle')
      .attr('dominant-baseline', 'middle')
      .attr('font-size', '40px')
      .attr('fill', 'var(--base-primary)')
      .attr('opacity', 0.8)
      .text(`${(progress).toFixed(0)}%`);
  }

  private updateProgressBar(newProgress: number): void {
    if (newProgress > 100) {
      newProgress = 100;
    }
    this.progress = newProgress;
    this.arc.endAngle(Math.PI * 2 * (newProgress / 100));
    this.progressBar
      .select('path')
      .attr('d', this.arc);
    this.progressBar
      .select('text')
      .text(`${(newProgress).toFixed(0)}%`);
  }
}