import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { untilDestroyed } from '@ngneat/until-destroy';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { WebNodeService } from '@core/services/web-node.service';
import { any, GlobalErrorHandlerService } from '@openmina/shared';
import { NgClass, NgForOf, NgIf, NgOptimizedImage } from '@angular/common';
import { Router } from '@angular/router';
import { getFirstFeature } from '@shared/constants/config';
import { animate, style, transition, trigger } from '@angular/animations';
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
}

@Component({
	selector: 'mina-web-node-initialization',
	templateUrl: './web-node-initialization.component.html',
	styleUrls: ['./web-node-initialization.component.scss'],
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
		trigger('messageChange', [
			transition('* => *', [
				style({ opacity: 0, transform: 'translateY(-10px)' }),
				animate('300ms ease-out', style({ opacity: 1, transform: 'translateY(0)' })),
			]),
		]),
	],
})
export class WebNodeInitializationComponent extends StoreDispatcher implements OnInit, AfterViewInit {

	protected readonly WebNodeStepStatus = WebNodeStepStatus;
	readonly loading: WebNodeLoadingStep[] = [
		{ name: 'Setting up browser for Web Node', loaded: false, status: WebNodeStepStatus.LOADING },
		{ name: 'Getting ready to produce blocks', loaded: false, status: WebNodeStepStatus.PENDING },
		{ name: 'Connecting directly to Mina network', loaded: false, status: WebNodeStepStatus.PENDING },
	];
	loadingMessage: string = '';
	downloadingMessage: string = '';
	ready: boolean = false;
	hasError: boolean = false;
	hasWarn: boolean = false;
	errors: string[] = [];

	private stepsPercentages: number[];
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
		window.dispatchEvent(new CustomEvent('startWebNode'));
		any(window).testtest = [];
		this.stepsPercentages = this.getStepPercentages();
		this.fetchProgress();
		this.listenToErrorIssuing();
		this.checkWebNodeProgress();
		this.fetchPeersInformation();
	}

	ngAfterViewInit(): void {
		this.buildProgressBar();
	}

	private checkWebNodeProgress(): void {
		this.webNodeService.webnodeProgress$.pipe(untilDestroyed(this)).subscribe((state: string) => {
			if (state === 'Loaded') {
				this.updateLoadingMessage('~5 seconds left');
				setTimeout(() => {
					if (!this.hasError && this.loading[1].status !== WebNodeStepStatus.DONE) {
						this.updateLoadingMessage('Slower than usual');
						this.hasWarn = true;
						this.detect();
					}
				}, 5000);
				this.loading[0].loaded = true;
				this.loading[0].status = WebNodeStepStatus.DONE;
				this.loading[1].status = WebNodeStepStatus.LOADING;
				this.advanceProgressFor2ndStep();
			} else if (state === 'Started') {
				this.updateLoadingMessage('~3 seconds left');
				setTimeout(() => {
					if (!this.hasError && this.loading[2].status !== WebNodeStepStatus.DONE) {
						this.updateLoadingMessage('Slower than usual');
						this.hasWarn = true;
						this.detect();
					}
				}, 3500);
				clearInterval(this.secondStepInterval);
				this.loading[0].loaded = true;
				this.loading[1].loaded = true;
				this.loading[0].status = WebNodeStepStatus.DONE;
				this.loading[1].status = WebNodeStepStatus.DONE;
				this.loading[2].status = WebNodeStepStatus.LOADING;
				this.advanceProgressFor3rdStep();
			} else if (state === 'Connected') {
				this.updateLoadingMessage('Web Node is ready');
				clearInterval(this.thirdStepInterval);
				this.loading[0].status = WebNodeStepStatus.DONE;
				this.loading[1].status = WebNodeStepStatus.DONE;
				this.loading[2].status = WebNodeStepStatus.DONE;
				this.loading.forEach((step: WebNodeLoadingStep) => step.loaded = true);
				this.goToEndProgress();
			}
			this.detect();
		});
	}

	private getStepPercentages(): number[] {
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

	private async advanceProgressFor3rdStep(): Promise<void> {
		// second step is done, now we are working on the third step.
		// each second add 1% to the progress.
		// but the third step is only 10% of the total progress.
		// so never go above 10%. Stop at 9% if the third step is not done yet.

		const currentProgress = this.progress;
		const targetProgress = this.stepsPercentages[0] + this.stepsPercentages[1] - 1;
		// run fast 5 increments to reach the target progress
		const diff = targetProgress - currentProgress;

		for (let i = 0; i < diff; i++) {
			await new Promise(resolve => setTimeout(resolve, 25));
			this.updateProgressBar(currentProgress + i);
			this.detect();
		}

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
			if (progress <= 100) {
				progress += 1;
				this.updateProgressBar(progress);
			} else {
				clearInterval(interval);
			}
		}, 30);
	}

	private fetchPeersInformation(): void {
		timer(0, 1000).pipe(
			switchMap(() => this.webNodeService.peers$),
			untilDestroyed(this),
		).subscribe();
	}

	private updateLoadingMessage(message: string): void {
		if (this.hasError || this.hasWarn) {
			return;
		}
		this.loadingMessage = message;
	}

	private listenToErrorIssuing(): void {
		this.errorHandler.errors$
			.pipe(filter(errors => !!errors.length), untilDestroyed(this))
			.subscribe((error: string) => {
				this.errors.push(error);
				this.loadingMessage = error;
				this.hasError = true;
				this.markErrorOnD3();

				this.detect();
			});
	}

	private fetchProgress(): void {
		FileProgressHelper.progress$.pipe(
			filter(Boolean),
			untilDestroyed(this),
		).subscribe((progress) => {
			this.downloadingMessage = `Downloading ${(progress.downloaded / 1e6).toFixed(1)} of ${(progress.totalSize / 1e6).toFixed(1)} MB`;
			if (this.svg) {
				const totalProgress = (progress.progress * this.stepsPercentages[0]) / 100;
				this.updateProgressBar(totalProgress);
			}
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
		const barWidth = 4;
		const progress = 0;

		this.svg = d3
			.select(this.chartContainer.nativeElement)
			.append('svg')
			.attr('width', width)
			.attr('height', height);

		this.progressBar = this.svg.append('g')
			.attr('transform', `translate(${width / 2}, ${height / 2})`);

		const radius = Math.min(width, height) / 2 - barWidth;

		this.progressBar.append('circle')
			.attr('r', radius)
			.attr('fill', 'none')
			.attr('stroke', 'var(--base-tertiary2)')
			.attr('stroke-width', 1);

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
			.attr('x2', '75%')
			.attr('y1', '25%')
			.attr('y2', '0%');

		gradient.append('stop')
			.attr('offset', '8%')
			.attr('stop-color', '#57d7ff');
		gradient.append('stop')
			.attr('offset', '60%')
			.attr('stop-color', '#fda2ff');
		gradient.append('stop')
			.attr('offset', '100%')
			.attr('stop-color', '#ff833d');

		this.progressBar.append('text')
			.attr('text-anchor', 'middle')
			.attr('alignment-baseline', 'central')
			.attr('dominant-baseline', 'central')
			.attr('font-size', '40px')
			.attr('fill', 'var(--base-primary)')
			.attr('opacity', 0.8)
			.attr('dx', '-.2em')
			.text((progress).toFixed(0));
		this.progressBar.append('text')
			.attr('class', 'symbol')
			.attr('text-anchor', 'middle')
			.attr('alignment-baseline', 'central')
			.attr('dominant-baseline', 'central')
			.attr('font-size', '20px')
			.attr('fill', 'var(--base-tertiary)')
			.attr('opacity', 0.8)
			.attr('dy', '-.3em')
			.attr('dx', '1.5em')
			.text('%');
	}

	private updateProgressBar(newProgress: number): void {
		any(window).testtest.push(newProgress);
		if (newProgress >= 100) {
			this.ready = true;
			newProgress = 100;
			this.detect();
		}
		this.progress = newProgress;
		this.arc.endAngle(Math.PI * 2 * (newProgress / 100));
		this.progressBar
			.select('path')
			.attr('d', this.arc);
		this.progressBar
			.select('text')
			.text((newProgress).toFixed(0));
		const numberOfDigits = newProgress.toFixed(0).length + 1;
		this.progressBar.select('.symbol')
			.attr('dx', `${numberOfDigits * 0.45}em`)
			.text('%');
	}

	private markErrorOnD3(): void {
		this.progressBar.select('path').attr('fill', 'var(--warn-primary)');
		this.progressBar.select('text').attr('fill', 'var(--warn-primary)');
		this.progressBar.select('.symbol').attr('fill', 'var(--warn-primary)');
		this.progressBar.select('path').attr('opacity', 0.4);
		this.progressBar.select('text').attr('opacity', 0.8);
		this.progressBar.select('.symbol').attr('opacity', 0.4);
	}
}
