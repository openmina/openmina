import { AfterViewInit, Directive, ElementRef, EventEmitter, HostListener, Output } from '@angular/core';
import { fromEvent } from 'rxjs';
import { debounceTime } from 'rxjs/operators';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';

export type ArrowHide = {
  showLeftArrow: boolean,
  showRightArrow: boolean,
};

@UntilDestroy()
@Directive({
  selector: '[minaHorizontalMenu]',
  standalone: true,
})
export class HorizontalMenuDirective implements AfterViewInit {

  @Output() arrows: EventEmitter<ArrowHide> = new EventEmitter<ArrowHide>();

  constructor(private element: ElementRef<HTMLDivElement>) { }

  ngAfterViewInit(): void {
    this.listenToWindowResize();
    this.listenToElWidthChange();
    this.listenToTouchEvents();
  }

  private listenToWindowResize(): void {
    fromEvent(window, 'resize').pipe(
      debounceTime(500),
      untilDestroyed(this),
    ).subscribe(() => this.checkArrows());
  }

  private listenToElWidthChange(): void {
    const widthChange = 'width-change';
    const resizeObserver = new ResizeObserver((entries: ResizeObserverEntry[]) => {
      for (const entry of entries) {
        const widthChangeEvent = new CustomEvent(widthChange);
        entry.target.dispatchEvent(widthChangeEvent);
      }
    });
    resizeObserver.observe(this.el);

    fromEvent(this.el, widthChange).pipe(
      debounceTime(500),
      untilDestroyed(this),
    ).subscribe(() => this.checkArrows());
  }

  private listenToTouchEvents(): void {
    let startX = 0;
    let currentX = 0;
    let diff = 0;

    fromEvent(this.el, 'touchstart')
      .pipe(untilDestroyed(this))
      .subscribe((event: any) => {
        startX = event.touches[0].pageX;
        currentX = this.el.scrollLeft;
      });

    fromEvent(this.el, 'touchmove')
      .pipe(untilDestroyed(this))
      .subscribe((event: any) => {
        diff = startX - event.touches[0].pageX;
        this.el.scrollLeft = currentX + diff;
        event.preventDefault();
      });

    fromEvent(this.el, 'touchend')
      .pipe(untilDestroyed(this))
      .subscribe(() => this.checkArrows());
  }

  @HostListener('scroll', ['$event'])
  private checkArrows(): void {
    const response: ArrowHide = {
      showLeftArrow: false,
      showRightArrow: false,
    };
    response.showLeftArrow = this.el.scrollLeft > 0;
    response.showRightArrow = (Math.ceil(this.el.scrollLeft) + 5) < (this.el.scrollWidth - this.el.clientWidth);
    this.arrows.emit(response);
  }

  private get el(): HTMLDivElement {
    return this.element.nativeElement;
  }
}
