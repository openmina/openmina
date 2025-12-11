import { ChangeDetectionStrategy, Component, ElementRef, Input, TemplateRef, ViewChild } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ArrowHide, HorizontalMenuDirective } from './horizontal-menu.directive';
import { ManualDetection } from '../../base-classes/manual-detection.class';
import { OpenminaEagerSharedModule } from '../../openmina-eager-shared.module';
import { REQUIRED } from '../../constants/angular';


@Component({
    selector: 'mina-horizontal-menu',
    templateUrl: './horizontal-menu.component.html',
    styleUrls: ['./horizontal-menu.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    imports: [OpenminaEagerSharedModule, CommonModule, HorizontalMenuDirective],
    host: { class: 'h-100 w-100 flex-column' }
})
export class HorizontalMenuComponent extends ManualDetection {

  @Input(REQUIRED) template: TemplateRef<any>;
  @Input() showNav: boolean = true;
  @Input() clz: string | string[];
  @Input() distance = 250;
  @Input() scrollSpeed = 100;

  showLeftArrow: boolean = false;
  showRightArrow: boolean = false;

  @ViewChild('hScroll') private scrollWrapper: ElementRef;

  listenToItemsScroll(e: ArrowHide): void {
    this.showLeftArrow = e.showLeftArrow;
    this.showRightArrow = e.showRightArrow;
  }

  scrollLeft(): void {
    const leftArrow = this.scrollWrapper.nativeElement;
    const scrollLeft = leftArrow.scrollLeft;
    const distance = scrollLeft - this.distance;
    this.scroll(distance);
  }

  scrollRight(): void {
    const listWrapper = this.scrollWrapper.nativeElement;
    const scrollLeft = listWrapper.scrollLeft;
    const distance = scrollLeft + this.distance;
    this.scroll(distance);
  }

  private scroll(distance: number): void {
    const listWrapper = this.scrollWrapper.nativeElement;
    listWrapper.scrollTo({ behavior: 'smooth', left: distance });
  }

  public checkView(): void {
    if (this.scrollWrapper.nativeElement.scrollWidth > this.scrollWrapper.nativeElement.clientWidth) {
      this.showRightArrow = true;
    } else {
      this.showRightArrow = false;
    }
    this.detect();
  }
}
