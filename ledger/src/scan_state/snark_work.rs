pub mod spec {
    use crate::scan_state::{
        scan_state::transaction_snark::{LedgerProof, Statement},
        transaction_logic::transaction_witness::TransactionWitness,
    };

    pub enum Work {
        Transition((Box<Statement<()>>, TransactionWitness)),
        Merge(Box<(Statement<()>, Box<(LedgerProof, LedgerProof)>)>),
    }
}

#[cfg(test)]
mod tests {
    use mina_p2p_messages::binprot::{
        self,
        macros::{BinProtRead, BinProtWrite},
    };
    use mina_p2p_messages::v2::{
        MinaBaseTransactionStatusStableV2, MinaTransactionTransactionStableV2,
        SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse,
        SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances,
    };

    use binprot::{BinProtRead, BinProtWrite};

    type SnarkWorkSpec = SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances;

    /// External worker input.
    #[derive(Debug, BinProtRead, BinProtWrite)]
    pub enum ExternalSnarkWorkerRequest {
        /// Queries worker for readiness, expected reply is `true`.
        AwaitReadiness,
        /// Commands worker to start specified snark job, expected reply is `ExternalSnarkWorkerResult`[ExternalSnarkWorkerResult].
        PerformJob(SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse),
    }

    // fn read_input<R: std::io::Read>(
    //     mut r: R,
    // ) -> (NonZeroCurvePoint, CurrencyFeeStableV1, SnarkWorkSpec) {
    //     let SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse(Some((
    //         SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0 { instances, fee },
    //         public_key,
    //     ))) = SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse::binprot_read(&mut r)
    //         .expect("cannot read work spec")
    //     else {
    //         unreachable!("incorrect work spec");
    //     };

    //     (public_key, fee, instances)
    // }

    // fn read_input<R: std::io::Read>(
    //     mut r: R,
    // ) {
    //     let value: SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse = SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse::binprot_read(&mut r)
    //         .expect("cannot read work spec");

    //     let request = ExternalSnarkWorkerRequest::PerformJob(value);

    //     let mut file = std::fs::File::create("/tmp/request.bin").unwrap();
    //     ExternalSnarkWorkerRequest::binprot_write(&request, &mut file).unwrap();
    //     file.sync_all().unwrap();

    //     // (public_key, fee, instances)
    // }

    fn write_binprot<T: BinProtWrite, W: std::io::Write>(spec: T, mut w: W) {
        let mut buf = Vec::new();
        spec.binprot_write(&mut buf).unwrap();
        let len = (buf.len() as u64).to_le_bytes();
        w.write_all(&len).unwrap();
        w.write_all(&buf).unwrap();
    }

    fn read_binprot<T, R>(mut r: R) -> T
    where
        T: BinProtRead,
        R: std::io::Read,
    {
        use std::io::Read;

        let mut len_buf = [0; std::mem::size_of::<u64>()];
        r.read_exact(&mut len_buf).unwrap();
        let len = u64::from_le_bytes(len_buf);

        let mut buf = Vec::with_capacity(len as usize);
        let mut r = r.take(len);
        r.read_to_end(&mut buf).unwrap();

        let mut read = buf.as_slice();
        T::binprot_read(&mut read).unwrap()
    }

    fn read_input<R: std::io::Read>(mut r: R) {
        let v: ExternalSnarkWorkerRequest = read_binprot(&mut r);
        // let value: ExternalSnarkWorkerRequest = ExternalSnarkWorkerRequest::binprot_read(&mut r)
        //     .expect("cannot read work spec");

        println!("OK");

        dbg!(v);

        // let request = ExternalSnarkWorkerRequest::PerformJob(value);

        // let mut file = std::fs::File::create("/tmp/request.bin").unwrap();
        // ExternalSnarkWorkerRequest::binprot_write(&request, &mut file).unwrap();
        // file.sync_all().unwrap();

        // (public_key, fee, instances)
    }

    // #[test]
    // fn snark_work() {
    //     // const DATA: &[u8] = include_bytes!("/tmp/input.bin");
    //     const DATA: &[u8] = include_bytes!("/tmp/spec1-header.bin");
    //     // const DATA: &[u8] = include_bytes!("/tmp/spec1.bin");
    //     let mut r = DATA;
    //     read_input(&mut r);
    //     // let (public_key, fee, instances) = read_input(&mut r);

    //     // dbg!(instances);
    //     // dbg!(&public_key, fee);
    // }

    #[test]
    fn snark_work2() {
        let Ok(r) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("requests_rampup4.bin"),
        ) else {
            return;
        };
        // const DATA: &[u8] = include_bytes!("/tmp/input.bin");
        // const DATA: &[u8] = include_bytes!("/tmp/requests.bin");
        // // const DATA: &[u8] = include_bytes!("/tmp/spec1.bin");
        // let mut r = DATA;
        let mut r = r.as_slice();

        let requests =
            Vec::<SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse>::binprot_read(&mut r).unwrap();

        dbg!(requests.len());

        let mut good = Vec::with_capacity(requests.len());

        for (index, req) in requests.iter().enumerate() {
            let index: usize = index;
            let req: &SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse = req;

            let Some((a, _prover)) = &req.0 else { panic!() };

            let work = match &a.instances {
                SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances::One(w) => w,
                SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances::Two(_) => todo!(),
            };

            let (_stmt, witness) = match work {
                mina_p2p_messages::v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Transition(stmt, witness) => (stmt, witness),
                mina_p2p_messages::v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Merge(_) => todo!(),
            };

            if matches!(witness.status, MinaBaseTransactionStatusStableV2::Failed(_)) {
                continue;
            }

            match &witness.transaction {
                MinaTransactionTransactionStableV2::Command(cmd) => match &**cmd {
                    mina_p2p_messages::v2::MinaBaseUserCommandStableV2::SignedCommand(_) => {
                        eprintln!("[{}] signed", index)
                    }
                    mina_p2p_messages::v2::MinaBaseUserCommandStableV2::ZkappCommand(z) => {
                        eprintln!("[{}] zkapp", index);
                        eprintln!("zkapp {:#?}", z);
                    }
                },
                MinaTransactionTransactionStableV2::FeeTransfer(_) => {
                    eprintln!("[{}] fee_transfer", index)
                }
                MinaTransactionTransactionStableV2::Coinbase(_) => {
                    eprintln!("[{}] coinbase", index)
                }
            }

            // if !matches!(
            //     witness.transaction,
            //     MinaTransactionTransactionStableV2::FeeTransfer(_)
            // ) {
            //     continue;
            // }

            let is_good = match &witness.transaction {
                MinaTransactionTransactionStableV2::Command(cmd) => {
                    match &**cmd {
                        mina_p2p_messages::v2::MinaBaseUserCommandStableV2::SignedCommand(cmd) => {
                            match &cmd.payload.body {
                                mina_p2p_messages::v2::MinaBaseSignedCommandPayloadBodyStableV2::Payment(_) => false,
                                mina_p2p_messages::v2::MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(_) => false,
                            }
                        },
                        mina_p2p_messages::v2::MinaBaseUserCommandStableV2::ZkappCommand(_) => true,
                    }
                },
                MinaTransactionTransactionStableV2::FeeTransfer(_) => false,
                MinaTransactionTransactionStableV2::Coinbase(_) => false,
            };

            if is_good {
                good.push(req.clone());
            }

            // match wi

            // eprintln!("is_zkapp={:?}", is_zkapp);
            // dbg!(is_zkapp);

            // dbg!(&witness.transaction, &witness.status);
        }

        dbg!(good.len());
        // dbg!(&good[0]);

        for (index, value) in good.iter().enumerate().take(10) {
            let value = ExternalSnarkWorkerRequest::PerformJob(value.clone());

            let mut file =
                std::fs::File::create(format!("/tmp/zkapp_{}_rampup4.bin", index)).unwrap();
            write_binprot(value, &mut file);
            file.sync_all().unwrap();
        }

        println!("OK");

        // dbg!(&requests[0]);

        // read_input(&mut r);
        // let (public_key, fee, instances) = read_input(&mut r);

        // dbg!(instances);
        // dbg!(&public_key, fee);
    }
}
