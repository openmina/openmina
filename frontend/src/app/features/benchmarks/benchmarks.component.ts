import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { BenchmarksWalletsService } from '@benchmarks/wallets/benchmarks-wallets.service';
import { ONE_BILLION } from '@openmina/shared';

@Component({
  selector: 'mina-benchmarks',
  templateUrl: './benchmarks.component.html',
  styleUrls: ['./benchmarks.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class BenchmarksComponent implements OnInit {

  constructor(private service: BenchmarksWalletsService) { }

  ngOnInit(): void {
    // setTimeout(() => {
    //   this.service.sendOneTx({
    //     from: 'B62qn3Ru1pK9jPT77wSGTdfQHF9rAMsEFuaGh3Ut4dLVXZanLXRtbGK',
    //     to: 'B62qmQak79sp14Amh2nq9oGNxhAgqQPwoKN4WbtD8h2ptzGj9WmotDy',
    //     amount: '2000000000',
    //     fee: ONE_BILLION.toString(),
    //     memo: 'test TEO',
    //     nonce: '0',
    //     validUntil: '4294967295',
    //     privateKey: 'EKEVVWtHG9e5yXcdNh5Up9P9v5MSXbo8XB53atRqJripggxJai6z',
    //   }).subscribe();
    // }, 3000);
    // this.store.dispatch<AppChangeSubMenus>({ type: APP_CHANGE_SUB_MENUS, payload: [Routes.WALLETS/*, Routes.TRANSACTIONS*/] });
  }
}
