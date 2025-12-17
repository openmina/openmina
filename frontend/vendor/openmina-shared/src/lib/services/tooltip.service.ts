import { Inject, Injectable } from '@angular/core';
import { ReplaySubject, Subject, take } from 'rxjs';
import { DOCUMENT } from '@angular/common';
import { MinaTooltipDirective } from '../directives/mina-tooltip.directive';
import { getLocalStorage } from '../helpers/browser.helper';

@Injectable({
  providedIn: 'root',
})
export class TooltipService {

  openTooltipsWithClipboardClick: number[] = [];
  justShowedTooltip: boolean = false;
  userExitedTooltip: boolean = false;
  openedTooltips: number = 0;
  private timeout: any;
  private popup: HTMLDivElement;

  readonly onTooltipChange$: ReplaySubject<boolean> = new ReplaySubject<boolean>();

  private readonly tooltipDisabledKey: string = 'tooltipDisabled';

  constructor(@Inject(DOCUMENT) private document: Document) {
    this.setInitialTooltipBehaviour();
    this.appendTooltipContainerToDOM();
    this.appendGraphTooltipToDOM();
  }

  private appendTooltipContainerToDOM(): void {
    const popup = this.document.createElement('div');
    popup.setAttribute('id', 'mina-tooltip');
    this.document.body.appendChild(popup);
    this.popup = popup;
  }

  private appendGraphTooltipToDOM(): void {
    const popup = this.document.createElement('div');
    popup.setAttribute('id', 'mina-graph-tooltip');
    this.document.body.appendChild(popup);
  }

  private setInitialTooltipBehaviour(): void {
    if (getLocalStorage()?.getItem(this.tooltipDisabledKey) === null) {
      getLocalStorage()?.setItem(this.tooltipDisabledKey, JSON.stringify(false));
    }
  }

  get graphTooltip(): HTMLDivElement {
    return this.document.getElementById('mina-graph-tooltip') as HTMLDivElement;
  }

  toggleTooltips(): void {
    const tooltipDisabled = this.getTooltipDisabledSetting();
    getLocalStorage()?.setItem(this.tooltipDisabledKey, JSON.stringify(!tooltipDisabled));
    this.onTooltipChange$.next(!tooltipDisabled);
  }

  getTooltipDisabledSetting(): boolean {
    return !!JSON.parse(getLocalStorage()?.getItem(this.tooltipDisabledKey));
  }

  onTooltipShow(): void {
    this.openedTooltips++;
    this.justShowedTooltip = true;
    this.userExitedTooltip = false;
    clearTimeout(this.timeout);
    this.timeout = setTimeout(() => {
      this.justShowedTooltip = false;
      this.openedTooltips = 0;
      if (this.userExitedTooltip) {
        MinaTooltipDirective.hideTooltip(this.popup);
      }
    }, 100);
  }
}
