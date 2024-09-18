import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { any, ManualDetection, MAX_WIDTH_700 } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';
import { AppSelectors } from '@app/app.state';
import { AppActions } from '@app/app.actions';
import { Observable, timer } from 'rxjs';
import { CONFIG } from '@shared/constants/config';
// import { AccountUpdate, declareMethods, Field, method, Mina, PrivateKey, SmartContract, State, state } from 'o1js';

// export class Square extends SmartContract {
//   @state(Field) num = State<Field>();
//
//   override init() {
//     super.init();
//     this.num.set(Field(3));
//   }
//
//   // @method
//   async update(square: Field) {
//     const currentState = this.num.get();
//     this.num.requireEquals(currentState);
//     square.assertEquals(currentState.mul(currentState));
//     this.num.set(square);
//   }
// }

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'd-block h-100 w-100' },
})
export class AppComponent extends ManualDetection implements OnInit {

  menu$: Observable<AppMenu> = this.store.select(AppSelectors.menu);
  subMenusLength: number = 0;
  hideToolbar: boolean = CONFIG.hideToolbar;

  constructor(private store: Store<MinaState>,
              private breakpointObserver: BreakpointObserver) {
    super();
    if (any(window).Cypress) {
      any(window).config = CONFIG;
      any(window).store = store;
    }
  }

  ngOnInit(): void {
    if (!this.hideToolbar && !CONFIG.hideNodeStats) {
      this.scheduleNodeUpdates();
    }
    this.listenToWindowResizing();
    // console.log('Start');
    // this.startZK();
    // console.log('Finish!');
  }


//   async startZK() {
//     //@ts-ignore
//     declareMethods(Square, { update: [Field] });
//
//     const useProof = false;
//
//     const Local = await Mina.LocalBlockchain({ proofsEnabled: useProof });
//     Mina.setActiveInstance(Local);
//
//     const deployerAccount = Local.testAccounts[0];
//     const deployerKey = deployerAccount.key;
//     const senderAccount = Local.testAccounts[1];
//     const senderKey = senderAccount.key;
// // ----------------------------------------------------
//
// // Create a public/private key pair. The public key is your address and where you deploy the zkApp to
//     const zkAppPrivateKey = PrivateKey.random();
//     const zkAppAddress = zkAppPrivateKey.toPublicKey();
//
// // create an instance of Square - and deploy it to zkAppAddress
//     const zkAppInstance = new Square(zkAppAddress);
//     const deployTxn = await Mina.transaction(deployerAccount, async () => {
//       AccountUpdate.fundNewAccount(deployerAccount);
//       await zkAppInstance.deploy();
//     });
//     await deployTxn.sign([deployerKey, zkAppPrivateKey]).send();
//
// // get the initial state of Square after deployment
//     const num0 = zkAppInstance.num.get();
//     console.log('state after init:', num0.toString());
//
// // ----------------------------------------------------
//
//     const txn1 = await Mina.transaction(senderAccount, async () => {
//       await zkAppInstance.update(Field(9));
//     });
//     await txn1.prove();
//     await txn1.sign([senderKey]).send();
//
//     const num1 = zkAppInstance.num.get();
//     console.log('state after txn1:', num1.toString());
//
// // ----------------------------------------------------
//
//     try {
//       const txn2 = await Mina.transaction(senderAccount, async () => {
//         await zkAppInstance.update(Field(75));
//       });
//       await txn2.prove();
//       await txn2.sign([senderKey]).send();
//     } catch (error: any) {
//       console.log(error.message);
//     }
//     const num2 = zkAppInstance.num.get();
//     console.log('state after txn2:', num2.toString());
//
// // ----------------------------------------------------
//
//     const txn3 = await Mina.transaction(senderAccount, async () => {
//       await zkAppInstance.update(Field(81));
//     });
//     await txn3.prove();
//     await txn3.sign([senderKey]).send();
//
//     const num3 = zkAppInstance.num.get();
//     console.log('state after txn3:', num3.toString());
//   }

  private scheduleNodeUpdates(): void {
    timer(1000, 5000).subscribe(() => this.store.dispatch(AppActions.getNodeDetails()));
  }

  private listenToWindowResizing(): void {
    this.breakpointObserver
      .observe(MAX_WIDTH_700)
      .subscribe((bs: BreakpointState) => {
        this.store.dispatch(AppActions.toggleMobile({ isMobile: bs.matches }));
      });
  }

  toggleMenu(): void {
    this.store.dispatch(AppActions.toggleMenuOpening());
  }

  onSubmenusLengthChange(length: number): void {
    this.subMenusLength = length;
  }
}
