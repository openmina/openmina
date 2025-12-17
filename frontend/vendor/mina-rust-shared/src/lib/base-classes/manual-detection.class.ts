import { ChangeDetectorRef, inject } from '@angular/core';

export abstract class ManualDetection {

  private changeDetectorRef: ChangeDetectorRef = inject<ChangeDetectorRef>(ChangeDetectorRef);

  detect(): void {
    this.changeDetectorRef.detectChanges();
  };

  mark(): void {
    this.changeDetectorRef.markForCheck();
  }
}
