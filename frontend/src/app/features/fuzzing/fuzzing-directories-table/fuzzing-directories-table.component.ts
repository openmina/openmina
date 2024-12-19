import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { selectFuzzingActiveDirectory, selectFuzzingDirectories } from '@fuzzing/fuzzing.state';
import { FuzzingSetActiveDirectory } from '@fuzzing/fuzzing.actions';
import { filter, take } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { FuzzingDirectory } from '@shared/types/fuzzing/fuzzing-directory.type';
import { getMergedRoute, MergedRoute, TableColumnList } from '@openmina/shared';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-fuzzing-directories-table',
  templateUrl: './fuzzing-directories-table.component.html',
  styleUrls: ['./fuzzing-directories-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100 border-right' },
})
export class FuzzingDirectoriesTableComponent extends MinaTableRustWrapper<FuzzingDirectory> implements OnInit {

  protected readonly tableHeads: TableColumnList<FuzzingDirectory> = [
    { name: 'datetime' },
    { name: 'directory name' },
  ];

  directories: FuzzingDirectory[] = [];
  activeDirectory: FuzzingDirectory;

  private pathFromRoute: string;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToRouteChange();
    this.listenToDirectories();
    this.listenToActiveDirectory();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [165, 165];
    this.table.propertyForActiveCheck = 'fullName';
    this.table.trackByFn = (_: number, item: FuzzingDirectory) => item.fullName;
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['dir'] && this.directories.length === 0) {
        this.pathFromRoute = route.params['dir'];
      }
    }, take(1));
  }

  private listenToDirectories(): void {
    this.select(selectFuzzingDirectories, (directories: FuzzingDirectory[]) => {
      this.directories = directories;
      this.table.rows = directories;
      if (this.pathFromRoute) {
        const payload = directories.find(dir => dir.fullName === this.pathFromRoute);
        delete this.pathFromRoute;
        if (payload) {
          this.dispatch(FuzzingSetActiveDirectory, payload);
        }
      } else if (!this.activeDirectory) {
        this.onRowClick(directories[0]);
      }
      this.table.detect();
      this.detect();
    }, filter(d => d.length > 0));
  }

  private listenToActiveDirectory(): void {
    this.select(selectFuzzingActiveDirectory, (directory: FuzzingDirectory) => {
      this.activeDirectory = directory;
      this.table.activeRow = directory;
      this.table.detect();
      this.detect();
    }, filter(directory => this.activeDirectory !== directory));
  }

  protected override onRowClick(dir: FuzzingDirectory): void {
    if (this.activeDirectory !== dir) {
      this.dispatch(FuzzingSetActiveDirectory, dir);
    }
    this.router.navigate([Routes.FUZZING, this.activeDirectory.fullName]);
  }
}
