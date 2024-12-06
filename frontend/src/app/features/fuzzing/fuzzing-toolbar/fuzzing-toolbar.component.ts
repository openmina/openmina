import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectFuzzingFiles } from '@fuzzing/fuzzing.state';
import { FuzzingFile } from '@shared/types/fuzzing/fuzzing-file.type';
import { FormBuilder, FormGroup } from '@angular/forms';
import { untilDestroyed } from '@ngneat/until-destroy';
import { FuzzingFilterFiles } from '@fuzzing/fuzzing.actions';
import { debounceTime } from 'rxjs';

@Component({
  selector: 'mina-fuzzing-toolbar',
  templateUrl: './fuzzing-toolbar.component.html',
  styleUrls: ['./fuzzing-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row align-center h-xl pl-12 border-bottom' },
})
export class FuzzingToolbarComponent extends StoreDispatcher implements OnInit {

  overallCoverage: number;
  coverageColor: string;
  formGroup: FormGroup;

  constructor(private fb: FormBuilder) { super(); }

  ngOnInit(): void {
    this.initForm();
    this.listenToFiles();
  }

  private initForm(): void {
    this.formGroup = this.fb.group({
      search: [''],
    });

    this.formGroup.get('search').valueChanges.pipe(
      untilDestroyed(this),
      debounceTime(150),
    ).subscribe((value: string) => {
      this.dispatch(FuzzingFilterFiles, value.trim().toLowerCase());
    });
  }

  private listenToFiles(): void {
    this.select(selectFuzzingFiles, (files: FuzzingFile[]) => {
      this.overallCoverage = files.reduce((acc: number, file: FuzzingFile) => acc + file.coverage, 0) / files.length;
      this.coverageColor = this.overallCoverage > 80 ? 'success' : this.overallCoverage > 50 ? 'aware' : 'warn';
      this.detect();
    });

  }
}
