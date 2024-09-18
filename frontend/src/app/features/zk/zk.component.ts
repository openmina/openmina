import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { AccountUpdate, Field, method, Mina, PrivateKey, Provable, PublicKey, SmartContract, state, State } from 'o1js';

class HelloWorld extends SmartContract {}


export class Square extends SmartContract {
  @state(Field) num = State<Field>();

  override init() {
    super.init();
    this.num.set(Field(3));
  }

  @method
  async update(square: Field) {
    const currentState = this.num.get();
    this.num.requireEquals(currentState);
    square.assertEquals(currentState.mul(currentState));
    this.num.set(square);
  }
}

@Component({
  selector: 'mina-zk',
  templateUrl: './zk.component.html',
  styleUrls: ['./zk.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ZkComponent implements OnInit {

  ngOnInit() {
    console.log('Start');
    this.startZK();
    console.log('Finish!');
  }

  async startZK() {

    const useProof = false;

    const Local = await Mina.LocalBlockchain({ proofsEnabled: useProof });
    Mina.setActiveInstance(Local);

    const deployerAccount = Local.testAccounts[0];
    const deployerKey = deployerAccount.key;
    const senderAccount = Local.testAccounts[1];
    const senderKey = senderAccount.key;
// ----------------------------------------------------

// Create a public/private key pair. The public key is your address and where you deploy the zkApp to
    const zkAppPrivateKey = PrivateKey.random();
    const zkAppAddress = zkAppPrivateKey.toPublicKey();

// create an instance of Square - and deploy it to zkAppAddress
    const zkAppInstance = new Square(zkAppAddress);
    const deployTxn = await Mina.transaction(deployerAccount, async () => {
      AccountUpdate.fundNewAccount(deployerAccount);
      await zkAppInstance.deploy();
    });
    await deployTxn.sign([deployerKey, zkAppPrivateKey]).send();

// get the initial state of Square after deployment
    const num0 = zkAppInstance.num.get();
    console.log('state after init:', num0.toString());

// ----------------------------------------------------

    const txn1 = await Mina.transaction(senderAccount, async () => {
      await zkAppInstance.update(Field(9));
    });
    await txn1.prove();
    await txn1.sign([senderKey]).send();

    const num1 = zkAppInstance.num.get();
    console.log('state after txn1:', num1.toString());

// ----------------------------------------------------

    try {
      const txn2 = await Mina.transaction(senderAccount, async () => {
        await zkAppInstance.update(Field(75));
      });
      await txn2.prove();
      await txn2.sign([senderKey]).send();
    } catch (error: any) {
      console.log(error.message);
    }
    const num2 = zkAppInstance.num.get();
    console.log('state after txn2:', num2.toString());

// ----------------------------------------------------

    const txn3 = await Mina.transaction(senderAccount, async () => {
      await zkAppInstance.update(Field(81));
    });
    await txn3.prove();
    await txn3.sign([senderKey]).send();

    const num3 = zkAppInstance.num.get();
    console.log('state after txn3:', num3.toString());
  }
}
