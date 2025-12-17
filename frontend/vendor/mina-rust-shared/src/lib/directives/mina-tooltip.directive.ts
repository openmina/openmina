import { Directive, ElementRef, HostListener, Inject, Input, OnDestroy, OnInit } from '@angular/core';
import { DOCUMENT } from '@angular/common';
import { TooltipService } from '../services/tooltip.service';

const TOOLTIP_OFFSET = 8;
const PX = 'px';

export enum TooltipPosition {
  TOP = 'top',
  BOTTOM = 'bottom',
  LEFT = 'left',
  RIGHT = 'right',
}

@Directive({
    selector: '[tooltip]',
    standalone: false
})
export class MinaTooltipDirective implements OnInit, OnDestroy {

  @Input() tooltip: string | number = '';
  @Input() showDelay: number = 0;
  @Input() hideDelay: number = 0;
  @Input() tooltipDisabled: boolean = false;
  @Input() globalTooltip: boolean = true;
  @Input() cancelFormatting: boolean = false;
  @Input() maxWidth: number = 250;
  @Input() position: TooltipPosition = TooltipPosition.BOTTOM;
  @Input() html: boolean = true;
  @Input() noAnimation: boolean = false;

  private popup: HTMLDivElement;
  private timer: any;
  private cancelShowing: boolean = false;

  constructor(private el: ElementRef,
              private tooltipService: TooltipService,
              @Inject(DOCUMENT) private document: Document) {}

  ngOnInit(): void {
    this.popup = this.document.getElementById('mina-tooltip') as HTMLDivElement;
    if (this.globalTooltip) {
      this.tooltipDisabled = this.tooltipService.getTooltipDisabledSetting() || this.tooltipDisabled;
    }
    // this.tooltipService.onTooltipChange$
    //   .asObservable()
    //   .pipe(untilDestroyed(this))
    //   .subscribe(value => this.tooltipDisabled = value);
  }

  @HostListener('mouseenter')
  private onMouseEnter(): void {
    if (this.tooltipDisabled || !this.tooltip) {
      return;
    }
    this.timer = setTimeout(() => {
      if (!this.cancelShowing) {
        this.tooltipService.onTooltipShow();
        MinaTooltipDirective.showTooltip(
          this.popup,
          this.el.nativeElement,
          this.tooltip.toString(),
          this.maxWidth,
          this.position,
          this.html,
          this.noAnimation,
        );
        if (this.cancelFormatting) {
          this.popup.classList.add('cancel-formatting');
        }
      }
    }, this.showDelay);
  }

  @HostListener('mouseleave')
  private onMouseLeave(): void {
    if (this.timer) {
      clearTimeout(this.timer);
    }
    this.tooltipService.userExitedTooltip = true;
    if (!this.tooltipService.justShowedTooltip || this.tooltipService.openedTooltips === 1) {
      setTimeout(() => { MinaTooltipDirective.hideTooltip(this.popup); }, this.hideDelay);
    }
  }

  ngOnDestroy(): void {
    this.cancelShowing = true;
    MinaTooltipDirective.hideTooltip(this.popup);
  }

  static showTooltip(popup: HTMLDivElement,
                     nativeElement: HTMLElement,
                     message: string,
                     maxWidth: number,
                     position: TooltipPosition,
                     html: boolean = true,
                     noAnimation: boolean = false): void {
    if (html) {
      popup.innerHTML = message;
    } else {
      popup.innerText = message;
    }
    const boundingClientRect = nativeElement.getBoundingClientRect();
    let x = boundingClientRect.left + (nativeElement.offsetWidth / 2) - (popup.offsetWidth / 2);
    let y = boundingClientRect.top + nativeElement.offsetHeight + TOOLTIP_OFFSET;
    let animationName = '';

    switch (position) {
      case TooltipPosition.TOP:
        animationName = TooltipPosition.BOTTOM;
        y = boundingClientRect.top - popup.offsetHeight - TOOLTIP_OFFSET;
        if (y < 0) {
          y = boundingClientRect.top + nativeElement.offsetHeight + TOOLTIP_OFFSET;
          animationName = TooltipPosition.TOP;
        }
        break;
      case TooltipPosition.BOTTOM:
        animationName = TooltipPosition.TOP;
        if ((y + popup.offsetHeight) > window.innerHeight) {
          y = boundingClientRect.top - popup.offsetHeight - TOOLTIP_OFFSET;
          animationName = TooltipPosition.BOTTOM;
        }
        break;
      case TooltipPosition.LEFT:
        animationName = TooltipPosition.RIGHT;
        x = boundingClientRect.left - popup.offsetWidth - TOOLTIP_OFFSET;
        y = boundingClientRect.top + (nativeElement.offsetHeight / 2) - (popup.offsetHeight / 2);
        if (x < 0) {
          x = boundingClientRect.left + nativeElement.offsetWidth + TOOLTIP_OFFSET;
          animationName = TooltipPosition.LEFT;
        }
        break;
      case TooltipPosition.RIGHT:
        animationName = TooltipPosition.LEFT;
        x = boundingClientRect.left + nativeElement.offsetWidth + TOOLTIP_OFFSET;
        y = boundingClientRect.top + (nativeElement.offsetHeight / 2) - (popup.offsetHeight / 2);
        if ((x + popup.offsetWidth) > window.innerWidth) {
          x = boundingClientRect.left - popup.offsetWidth - TOOLTIP_OFFSET;
          animationName = TooltipPosition.RIGHT;
        }
        break;
    }

    if ((x + popup.offsetWidth) > window.innerWidth) {
      x = window.innerWidth - popup.offsetWidth - TOOLTIP_OFFSET;
    } else if (x < 0) {
      x = TOOLTIP_OFFSET;
    }

    popup.style.maxWidth = maxWidth + PX;
    popup.style.top = y + PX;
    popup.style.left = x + PX;
    popup.style.animationName = 'tooltip-' + (noAnimation ? 'no-animation' : ('slide-' + animationName));
  }

  static hideTooltip(popup: HTMLDivElement): void {
    if (popup) {
      popup.setAttribute('style', 'top:-1000px;left-1000px');
    }
  }
}
