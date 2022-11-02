use ocaml_interop::*;

use crate::MessagesForNextStepProof;

ocaml_export! {
    fn rust_get_random_msg_step_proof(
        rt,
        validate_msg: OCamlRef<fn (OCamlBytes) -> ()>,
    ) -> OCaml<OCamlBytes> {
        let mut msg;
        // let mut bytes;
        let validate_msg = validate_msg.to_boxroot(rt);

        loop {
            msg = MessagesForNextStepProof::rand();
            // bytes = serde_binprot::to_vec(&msg).unwrap();

            // if validate_msg.try_call(rt, &bytes).is_ok() {
            //     break;
            // }
        }

        todo!()

        // println!("msg={:?}", account.id());
        // std::thread::sleep_ms(2000);

        // bytes.to_ocaml(rt)
    }
}
