import { Directive, ElementRef, EventEmitter, Inject, Input, OnInit, Output } from '@angular/core';
import {
  distinctUntilChanged,
  filter,
  fromEvent,
  map,
  merge,
  Observable,
  Subject,
  switchMap,
  takeUntil,
  tap,
  throttleTime
} from 'rxjs';
import { DOCUMENT } from '@angular/common';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import { getLocalStorage } from '../../helpers/browser.helper';

@UntilDestroy()
@Directive({
  selector: '[horizontalResize]',
  standalone: true,
})
export class HorizontalResizeDirective implements OnInit {

  @Input() minWidth: number | null;
  @Input() maxWidth: number | null;
  @Input() maxWidthElement: HTMLElement | null;
  @Input() localStorageKey: string;

  @Output() horizontalResize: Observable<number>;
  @Output() startResizing: EventEmitter<void> = new EventEmitter<void>();
  @Output() endResizing: Observable<MouseEvent>;

  private resizeInProgress: boolean;
  private windowResize$: Subject<number> = new Subject<number>();
  private maxCalculatedWidth: number;
  private initialWidthOnResize: number;

  constructor(@Inject(DOCUMENT) private readonly document: Document,
              @Inject(ElementRef) private readonly elementRef: ElementRef<HTMLElement>) {

    this.horizontalResize = merge(
      fromEvent<MouseEvent>(this.elementRef.nativeElement, 'mousedown')
        .pipe(
          tap((e: MouseEvent) => e.preventDefault()),
          switchMap(() => {
            this.startResizing.emit();
            this.resizeInProgress = true;
            const { width, left } = this.elementRef.nativeElement
              .parentElement.parentElement
              .getBoundingClientRect();
            this.initialWidthOnResize = this.elementRef.nativeElement.parentElement.offsetWidth;

            return fromEvent<MouseEvent>(this.document, 'mousemove').pipe(
              map(({ clientX }: { clientX: number }) => {
                const newValue = left + width - clientX;
                if (this.minWidth && this.minWidth > newValue) {
                  return this.minWidth;
                } else if (this.maxCalculatedWidth && this.maxCalculatedWidth < newValue) {
                  return this.maxCalculatedWidth;
                } else {
                  return newValue;
                }
              }),
              distinctUntilChanged(),
              takeUntil(fromEvent(this.document, 'mouseup')),
            );
          }),
        ),
      this.windowResize$.asObservable(),
    );

    this.endResizing = fromEvent<MouseEvent>(this.document, 'mouseup')
      .pipe(
        filter(() => this.resizeInProgress),
        tap(() => this.resizeInProgress = false),
        filter(() => this.initialWidthOnResize !== this.elementRef.nativeElement.parentElement.offsetWidth),
      );
  }

  ngOnInit(): void {
    this.maxCalculatedWidth = this.getMaxCalculatedWidth();

    fromEvent(window, 'resize')
      .pipe(
        untilDestroyed(this),
        throttleTime(50),
        distinctUntilChanged(),
      )
      .subscribe(() => {
        this.maxCalculatedWidth = this.getMaxCalculatedWidth();
        const currentWidth = Number(getLocalStorage()?.getItem(this.localStorageKey));
        if (this.maxCalculatedWidth < this.minWidth) {
          this.maxCalculatedWidth = this.minWidth;
          this.windowResize$.next(this.maxCalculatedWidth);
        } else if (this.maxCalculatedWidth < currentWidth) {
          this.windowResize$.next(this.maxCalculatedWidth);
        }
      });
  }

  private getMaxCalculatedWidth(): number {
    return this.maxWidth || (this.maxWidthElement.offsetWidth - 50);
  }
}
