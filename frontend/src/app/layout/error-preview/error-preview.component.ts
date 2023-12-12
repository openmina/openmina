import { ChangeDetectionStrategy, Component, ComponentRef, OnInit } from '@angular/core';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { ManualDetection } from '@openmina/shared';
import { selectErrorPreviewErrors } from '@error-preview/error-preview.state';
import { filter, take } from 'rxjs';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { ComponentPortal } from '@angular/cdk/portal';
import { ErrorListComponent } from '@error-preview/error-list/error-list.component';
import { MinaError } from '@shared/types/error-preview/mina-error.type';
import { MARK_ERRORS_AS_SEEN, MarkErrorsAsSeen } from '@error-preview/error-preview.actions';

@Component({
  selector: 'mina-error-preview',
  templateUrl: './error-preview.component.html',
  styleUrls: ['./error-preview.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ErrorPreviewComponent extends ManualDetection implements OnInit {

  errors: MinaError[] = [];
  newMessages: void[] = [];
  newError: MinaError;
  unreadErrors: boolean;
  openedOverlay: boolean;

  private overlayRef: OverlayRef;
  private errorListComponent: ComponentRef<ErrorListComponent>;

  constructor(private store: Store<MinaState>,
              private overlay: Overlay) { super(); }

  ngOnInit(): void {
    this.listenToNewErrors();
  }

  private listenToNewErrors(): void {
    this.store.select(selectErrorPreviewErrors)
      .pipe(filter(errors => !!errors.length))
      .subscribe((errors: MinaError[]) => {
        if (errors.length !== this.errors.length) {
          this.newError = errors[0];
          this.newMessages.push(void 0);
          setTimeout(() => {
            this.newMessages.pop();
            this.detect();
          }, 3000);
        }

        this.errors = errors;
        this.unreadErrors = errors.some(e => !e.seen);
        if (this.errorListComponent) {
          this.errorListComponent.instance.errors = errors;
          this.errorListComponent.instance.detect();
        }
        this.detect();
      });
  }

  openErrorList(anchor: HTMLSpanElement, event: MouseEvent): void {
    this.openedOverlay = true;
    if (this.overlayRef?.hasAttached()) {
      this.openedOverlay = false;
      this.overlayRef.detach();
      this.store.dispatch<MarkErrorsAsSeen>({ type: MARK_ERRORS_AS_SEEN });
      return;
    }

    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(anchor)
        .withPositions([{
          originX: 'start',
          originY: 'bottom',
          overlayX: 'start',
          overlayY: 'top',
          // offsetX: -10,
          offsetY: 14,
        }]),
    });
    event.stopPropagation();

    const portal = new ComponentPortal(ErrorListComponent);
    this.errorListComponent = this.overlayRef.attach<ErrorListComponent>(portal);
    this.errorListComponent.instance.errors = this.errors;
    this.errorListComponent.instance.onConfirm
      .pipe(take(1))
      .subscribe(() => {
        this.detachOverlay();

        if (this.unreadErrors) {
          this.store.dispatch<MarkErrorsAsSeen>({ type: MARK_ERRORS_AS_SEEN });
        }
      });
  }

  private detachOverlay(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
      this.openedOverlay = false;
      this.detect();
    }
  }
}
