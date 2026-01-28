import { Directive, ElementRef, EventEmitter, HostListener, Output } from '@angular/core';

@Directive({
    selector: '[clickOutside]',
    standalone: false
})
export class ClickOutsideDirective {

  @Output() clickOutside: EventEmitter<void> = new EventEmitter<void>();

  @HostListener('document:click', ['$event.target'])
  onClick(target: EventTarget | null): void {
    const clickedInside = this.elementRef.nativeElement.contains(target as Node);
    if (!clickedInside) {
      this.clickOutside.emit();
    }
  }

  constructor(private elementRef: ElementRef) { }

}
