import {
  AccountUpdate,
  Field,
  method,
  Mina,
  Permissions,
  PrivateKey,
  PublicKey,
  SmartContract,
  state,
  State,
} from 'o1js';

class Add extends SmartContract {
  @state(Field) num = State<Field>();

  override init(): void {
    this.account.provedState.requireEquals(this.account.provedState.get());
    this.account.provedState.get().assertFalse();

    super.init();
    this.num.set(Field(1));
  }

  @method
  async update() {
    const currentState = this.num.getAndRequireEquals();
    const newState = currentState.add(5);
    this.num.set(newState);
  }

  override async deploy() {
    super.deploy();
    this.account.permissions.set({
      ...Permissions.default(),
      setDelegate: Permissions.signature(),
      setPermissions: Permissions.signature(),
      setZkappUri: Permissions.signature(),
      setTokenSymbol: Permissions.signature(),
      incrementNonce: Permissions.signature(),
      setVotingFor: Permissions.signature(),
      setTiming: Permissions.signature(),
      send: Permissions.proof(),
      editState: Permissions.proof(),
      receive: Permissions.none(),
      access: Permissions.none(),
      editActionState: Permissions.proof(),
    });
  }
}

export interface ZkInput {
  payerPublicKey: string;
  payerPrivateKey: string;
  fee: number;
  nonce: string;
  memo?: string;
  accountUpdates: number;
}

export async function deployZkApp(graphQlUrl: string, input: ZkInput, updates: {
  next: (val: { step: string, duration: number }) => void
}): Promise<any> {
  console.log('----------- Sending ZkApp -----------');
  const network = Mina.Network(graphQlUrl);
  Mina.setActiveInstance(network);
  const pairs = Array.from({ length: input.accountUpdates }, () => {
    const randPrivateKey = PrivateKey.random();
    return {
      publicKey: randPrivateKey.toPublicKey(),
      privateKey: randPrivateKey,
    };
  });
  console.log(pairs.map((pair) => pair.privateKey.toBase58()));
  const zkApps: Add[] = pairs.map((pair) => new Add(pair.publicKey));

  let stepStartTime = performance.now();

  const updateStep = (step: string) => {
    const now = performance.now();
    if (step === 'Compiling') {
      updates.next({ step, duration: undefined });
      return;
    }
    let duration = Math.round((now - stepStartTime) / 1000 * 1000) / 1000;
    updates.next({ step, duration });
    console.log(`${step} (${duration}s)`);
    stepStartTime = now;
  };

  updateStep('Compiling');
  await Add.compile();
  updateStep('Compiled');

  const payerAccount = {
    sender: PublicKey.fromBase58(input.payerPublicKey),
    fee: input.fee * 1e9,
    nonce: Number(input.nonce),
    memo: input.memo,
  };
  let tx = await Mina.transaction(payerAccount, async () => {
    AccountUpdate.fundNewAccount(PublicKey.fromBase58(input.payerPublicKey), input.accountUpdates);
    for (const zkApp of zkApps) {
      await zkApp.deploy();
    }
  });

  updateStep('Deployed');

  await tx.prove();
  updateStep('Proved');

  await tx.sign([PrivateKey.fromBase58(input.payerPrivateKey), ...pairs.map((pair) => pair.privateKey)]);
  updateStep('Signed');

  return tx.safeSend().then((sentTx) => {
    updateStep('Sent');
    console.log(sentTx);
    console.log('----------- Done -----------');
    return sentTx;
  });
}

const deployedZkApps: string[] = [
  'EKEbTHeqQbq5zeFuspjVSoatEebrG7fJnz8CrXyP4aVAXzeD1Z6A',
  'EKFF1zZ4KUCZoe7GXHAPcfLdkGPgsYJ5RNtQvHMx8ndhY1pZttaa',
];

export async function updateZkApp(graphQlUrl: string, input: ZkInput, updates: {
  next: (val: { step: string, duration: number }) => void
}): Promise<any> {
  console.log('----------- Updating ZkApp -----------');
  const network = Mina.Network(graphQlUrl);
  Mina.setActiveInstance(network);
  const pairs = Array.from({ length: input.accountUpdates }, (_, i: number) => {
    const randPrivateKey = PrivateKey.fromBase58(deployedZkApps[i]);
    return {
      publicKey: randPrivateKey.toPublicKey(),
      privateKey: randPrivateKey,
    };
  });
  const zkApps: Add[] = pairs.map((pair) => new Add(pair.publicKey));

  let stepStartTime = performance.now();

  const updateStep = (step: string) => {
    const now = performance.now();
    if (step === 'Compiling') {
      updates.next({ step, duration: undefined });
      return;
    }
    let duration = Math.round((now - stepStartTime) / 1000 * 1000) / 1000;
    updates.next({ step, duration });
    console.log(`${step} (${duration}s)`);
    stepStartTime = now;
  };
  updateStep('Compiling');

  await Add.compile();
  updateStep('Compiled');

  const payerAccount = {
    sender: PublicKey.fromBase58(input.payerPublicKey),
    fee: input.fee * 1e9,
    nonce: Number(input.nonce),
    memo: input.memo,
  };
  let tx = await Mina.transaction(payerAccount, async () => {
    for (const zkApp of zkApps) {
      await zkApp.update();
    }
  });

  updateStep('Proved Check Even');

  await tx.prove();
  updateStep('Proved');

  await tx.sign([PrivateKey.fromBase58(input.payerPrivateKey), ...pairs.map((pair) => pair.privateKey)]);
  updateStep('Signed');

  return tx.safeSend().then((sentTx) => {
    updateStep('Sent');
    updateStep(null);
    console.log(sentTx);
    console.log('----------- Done -----------');
    return sentTx;
  });
}
