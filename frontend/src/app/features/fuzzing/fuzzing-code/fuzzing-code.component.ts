import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, Inject, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { FuzzingFile } from '@shared/types/fuzzing/fuzzing-file.type';
import { FuzzingFileDetails } from '@shared/types/fuzzing/fuzzing-file-details.type';
import { selectFuzzingActiveFile, selectFuzzingActiveFileDetails } from '@fuzzing/fuzzing.state';
import { debounceTime, filter, fromEvent, tap, zip } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { ActivatedRoute, Router } from '@angular/router';
import { FuzzingGetFileDetails } from '@fuzzing/fuzzing.actions';
import { untilDestroyed } from '@ngneat/until-destroy';
import { getMergedRoute, MergedRoute, MinaTooltipDirective, TooltipPosition } from '@openmina/shared';
import { DOCUMENT } from '@angular/common';

@Component({
  selector: 'mina-fuzzing-code',
  templateUrl: './fuzzing-code.component.html',
  styleUrls: ['./fuzzing-code.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 border-left flex-column' },
})
export class FuzzingCodeComponent extends StoreDispatcher implements OnInit, AfterViewInit {

  file: FuzzingFile;
  fileDetails: FuzzingFileDetails;
  codeHighlighted: boolean = true;
  link: string;
  activeLineFromUrl: number;

  private popup: HTMLDivElement;
  private lineToScroll: number;
  @ViewChild('codeHolder') private codeHolder: ElementRef<HTMLDivElement>;
  @ViewChild('codeContainer') private codeContainer: ElementRef<HTMLDivElement>;

  constructor(@Inject(DOCUMENT) private document: Document,
              private router: Router,
              private activatedRoute: ActivatedRoute) { super(); }

  ngOnInit(): void {
    this.listenToFileChanges();
  }

  ngAfterViewInit(): void {
    this.listenToMouseMove();
    this.listenToRouteChange();
    this.popup = this.document.getElementById('mina-tooltip') as HTMLDivElement;
  }

  closeSidePanel(): void {
    this.router.navigate([Routes.FUZZING]);
    this.dispatch(FuzzingGetFileDetails, undefined);
  }

  private listenToFileChanges(): void {
    zip(
      this.store.select(selectFuzzingActiveFile).pipe(
        filter(file => this.file !== file),
      ),
      this.store.select(selectFuzzingActiveFileDetails).pipe(
        filter(file => this.fileDetails !== file),
      ),
    )
      .pipe(untilDestroyed(this))
      .subscribe(([file, details]) => {
        this.file = file;
        this.fileDetails = details;
        this.codeHolder.nativeElement.scrollTo(0, 0);
        if (this.lineToScroll) {
          this.detect();
          this.codeHolder.nativeElement.scrollTo(0, (Number(this.lineToScroll) - 1) * 24);
          delete this.lineToScroll;
        }
        this.link = `${window.location.origin}${window.location.pathname}${window.location.hash}?line=`;
        this.detect();
      });
  }

  toggleCodeHighlighting(): void {
    this.codeHighlighted = !this.codeHighlighted;
  }

  private listenToMouseMove(): void {
    fromEvent(this.codeContainer.nativeElement, 'mousemove').pipe(
      untilDestroyed(this),
      tap(() => MinaTooltipDirective.hideTooltip(this.popup)),
      debounceTime(200),
    ).subscribe((ev: Event) => {
      const target = ev.target as HTMLSpanElement;
      if (target.hasAttribute('h')) {
        MinaTooltipDirective.showTooltip(this.popup, target, 'Hits: ' + target.getAttribute('h'), 500, TooltipPosition.BOTTOM);
      }
    });
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      const line = route.queryParams['line'];
      if (!this.activeLineFromUrl && line) {
        this.lineToScroll = Number(line);
      }
      this.activeLineFromUrl = Number(line);
    });
  }

  onRowClick(line: number): void {
    this.router.navigate([], { queryParams: { line: line.toString() }, queryParamsHandling: 'merge' });
  }
}
