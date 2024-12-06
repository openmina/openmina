import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { FuzzingFile } from '@shared/types/fuzzing/fuzzing-file.type';
import { selectFuzzingActiveDirectory, selectFuzzingActiveFile, selectFuzzingFiles, selectFuzzingFilesSorting } from '@fuzzing/fuzzing.state';
import { FuzzingGetFileDetails, FuzzingSort } from '@fuzzing/fuzzing.actions';
import { filter, take, timer } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { untilDestroyed } from '@ngneat/until-destroy';
import { FuzzingDirectory } from '@shared/types/fuzzing/fuzzing-directory.type';
import { getMergedRoute, MergedRoute, TableColumnList, TableSort } from '@openmina/shared';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-fuzzing-files-table',
  templateUrl: './fuzzing-files-table.component.html',
  styleUrls: ['./fuzzing-files-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class FuzzingFilesTableComponent extends MinaTableRustWrapper<FuzzingFile> implements OnInit {

  protected readonly tableHeads: TableColumnList<FuzzingFile> = [
    { name: 'coverage', sort: 'coverage' },
    { name: 'path', sort: 'path' },
  ];

  files: FuzzingFile[] = [];
  activeFile: FuzzingFile;
  currentSort: TableSort<FuzzingFile>;

  private pathFromRoute: string;
  private activeDirectory: FuzzingDirectory;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToRouteChange();
    this.listenToSortingChanges();
    this.listenToFiles();
    this.listenToActiveFile();
    this.listenToActiveDirectory();

    timer(0, 5000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        if (this.activeFile) {
          this.dispatch(FuzzingGetFileDetails, this.activeFile);
        }
      });
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [150, 430];
    this.table.propertyForActiveCheck = 'path';
    this.table.sortClz = FuzzingSort;
    this.table.sortSelector = selectFuzzingFilesSorting;
    this.table.trackByFn = (_: number, item: FuzzingFile) => item.path;
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['file'] && this.files.length === 0) {
        this.pathFromRoute = route.params['file'];
      }
    }, take(1));
  }

  private listenToFiles(): void {
    this.select(selectFuzzingFiles, (files: FuzzingFile[]) => {
      this.files = files;
      this.table.rows = files;
      if (files.length > 0 && this.pathFromRoute) {
        const payload = files.find(file => file.path === this.pathFromRoute);
        if (payload) {
          this.dispatch(FuzzingGetFileDetails, payload);
          this.table.detect();
          this.detect();
          this.scrollToElement();
          delete this.pathFromRoute;
          return;
        }
      }
      this.table.detect();
      this.detect();
    });
  }

  private listenToActiveFile(): void {
    this.select(selectFuzzingActiveFile, (file: FuzzingFile) => {
      this.activeFile = file;
      this.table.activeRow = file;
      this.table.detect();
      this.detect();
    }, filter(file => this.activeFile !== file));
  }

  private listenToActiveDirectory(): void {
    this.select(selectFuzzingActiveDirectory, (directory: FuzzingDirectory) => {
      this.activeDirectory = directory;
    }, filter(directory => this.activeDirectory !== directory));
  }

  private listenToSortingChanges(): void {
    this.select(selectFuzzingFilesSorting, sort => {
      this.currentSort = sort;
      this.detect();
    });
  }

  private scrollToElement(): void {
    if (!this.pathFromRoute) {
      return;
    }
    const topElements = Math.floor(this.table.virtualScroll.elementRef.nativeElement.offsetHeight / 2 / this.table.rowSize);
    const index = this.files.findIndex(file => file.path === this.pathFromRoute) - topElements;
    this.table.virtualScroll.scrollToIndex(index);
  }

  protected override onRowClick(file: FuzzingFile): void {
    if (this.activeFile?.path !== file.path) {
      this.dispatch(FuzzingGetFileDetails, file);
    }
    this.router.navigate([Routes.FUZZING, this.activeDirectory.fullName, file.path]);
  }
}
