import { Directive, ElementRef, HostListener, Inject, Input, OnInit } from '@angular/core';
import { Clipboard } from '@angular/cdk/clipboard';
import { DOCUMENT } from '@angular/common';
import { TooltipService } from '../services/tooltip.service';
import { MinaTooltipDirective, TooltipPosition } from './mina-tooltip.directive';

@Directive({
    selector: '[copyToClipboard]',
    standalone: false
})
export class CopyToClipboardDirective implements OnInit {

  @Input() copyToClipboard: string;

  private popup: HTMLDivElement;

  constructor(private el: ElementRef<HTMLElement>,
              @Inject(DOCUMENT) private document: Document,
              private tooltipService: TooltipService,
              private clipboard: Clipboard) { }

  ngOnInit(): void {
    this.popup = this.document.getElementById('mina-tooltip') as HTMLDivElement;
  }

  @HostListener('click', ['$event'])
  private onClick(event: MouseEvent): void {
    event.stopPropagation();
    this.clipboard.copy(this.copyToClipboard);
    this.tooltipService.openTooltipsWithClipboardClick.push(0);
    MinaTooltipDirective.showTooltip(this.popup, this.el.nativeElement, 'Copied to clipboard', 250, TooltipPosition.BOTTOM);

    setTimeout(() => {
      this.tooltipService.openTooltipsWithClipboardClick.pop();
      if (this.tooltipService.openTooltipsWithClipboardClick.length === 0) {
        MinaTooltipDirective.hideTooltip(this.popup);
      }
    }, 1500);
  }
}
