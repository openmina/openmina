import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  EventEmitter,
  Input,
  OnChanges,
  OnInit,
  Output,
  SimpleChanges,
  TemplateRef,
  ViewChild,
} from '@angular/core';
import { HorizontalResizeDirective } from './horizontal-resize.directive';
import { OpenminaSharedModule } from '../../openmina-shared.module';
import { CommonModule } from '@angular/common';
import { REQUIRED } from '../../constants/angular';
import { getLocalStorage } from '../../helpers/browser.helper';

@Component({
    selector: 'mina-horizontal-resizable-container',
    imports: [HorizontalResizeDirective, OpenminaSharedModule, CommonModule],
    templateUrl: './horizontal-resizable-container.component.html',
    styleUrls: ['./horizontal-resizable-container.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    host: { class: 'w-100 h-100 p-relative flex-row' }
})
export class HorizontalResizableContainerComponent implements OnInit, OnChanges {

  @Input() minWidth: number | null = null;
  @Input() maxWidth: number | null = null;
  @Input() maxWidthElement: HTMLElement | null = null;
  @Input(REQUIRED) localStorageKey: string;
  @Input(REQUIRED) show: boolean;
  @Input(REQUIRED) leftTemplate: TemplateRef<void>;
  @Input(REQUIRED) rightTemplate: TemplateRef<void>;
  @Output() endResizing: EventEmitter<void> = new EventEmitter<void>();

  width: number | null = null;

  private removedClass: boolean;

  @ViewChild('main', { static: true }) private main: ElementRef<HTMLElement>;
  @ViewChild('aside', { static: true }) private aside: ElementRef<HTMLElement>;

  ngOnInit(): void {
    const localStorageWidth = Number(getLocalStorage()?.getItem(this.localStorageKey));
    if (localStorageWidth) {
      this.onResize(localStorageWidth);
    } else {
      this.onResize(this.minWidth);
    }
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (this.show && !this.removedClass) {
      this.removedClass = true;
      this.aside.nativeElement.classList.remove('no-transition');
    }
    this.setDimensions();
  }

  onResize(width: number): void {
    this.width = width;
    getLocalStorage()?.setItem(this.localStorageKey, width.toString());
    this.setDimensions();
  }

  onStartResizing(): void {
    this.toggleMain();
  }

  onEndResizing(): void {
    this.endResizing.emit();
    this.toggleMain();
  }

  private toggleMain(): void {
    this.main.nativeElement.classList.toggle('no-transition');
  }

  private setDimensions(): void {
    this.aside.nativeElement.style.width = `${this.width}px`;
    if (this.show) {
      this.aside.nativeElement.style.right = '0';
      this.main.nativeElement.style.width = `calc(100% - ${this.width}px)`;
    } else {
      this.aside.nativeElement.style.right = window.innerWidth > 700 ? `-${this.width}px` : '-100%';
      this.main.nativeElement.style.width = '100%';
    }
  }
}
