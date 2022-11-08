use ocaml_interop::*;

use crate::ffi::util::*;
use crate::{MessagesForNextStepProof, MessagesForNextWrapProof};

ocaml_export! {
    fn rust_hash_message_for_next_step_proof(
        rt,
        bytes: OCamlRef<OCamlBytes>,
        ocaml_hash: OCamlRef<OCamlList<String>>,
    ) {
        let mut ocaml_hash_ref = rt.get(ocaml_hash);

        let mut ocaml_hash = Vec::with_capacity(2048);
        while let Some((head, tail)) = ocaml_hash_ref.uncons() {
            let limb: String = head.to_rust();
            let limb: i64 = limb.parse().unwrap();
            ocaml_hash.push(limb as u64);
            ocaml_hash_ref = tail;
        }

        eprintln!("RESULT_OCAML={:?}", ocaml_hash);

        let bytes = rt.get(bytes);

        let msg: MessagesForNextStepProof = deserialize(bytes.as_bytes());
        let rust_hash = msg.hash();

        eprintln!("RESULT_RUST_={:?}", rust_hash);

        assert_eq!(&rust_hash[..], ocaml_hash);

        OCaml::unit()
    }

    fn rust_get_random_message(
        rt,
        validate_msg: OCamlRef<fn (OCamlBytes) -> ()>,
    ) -> OCaml<OCamlBytes> {
        let msg = MessagesForNextStepProof::rand();
        let bytes = serialize(&msg);

        // let s = bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("");
        // eprintln!("BYTES={:?}", s);

        bytes.to_ocaml(rt)
    }

    fn rust_get_random_wrap_message(
        rt,
        validate_msg: OCamlRef<fn (OCamlBytes) -> ()>,
    ) -> OCaml<OCamlBytes> {
        let msg = MessagesForNextWrapProof::rand();
        let bytes = serialize(&msg);

        // let s = bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("");
        // eprintln!("BYTES={:?}", s);

        bytes.to_ocaml(rt)
    }

    fn rust_hash_message_for_next_wrap_proof(
        rt,
        bytes: OCamlRef<OCamlBytes>,
        ocaml_hash: OCamlRef<OCamlList<String>>,
    ) {
        let mut ocaml_hash_ref = rt.get(ocaml_hash);

        let mut ocaml_hash = Vec::with_capacity(2048);
        while let Some((head, tail)) = ocaml_hash_ref.uncons() {
            let limb: String = head.to_rust();
            let limb: i64 = limb.parse().unwrap();
            ocaml_hash.push(limb as u64);
            ocaml_hash_ref = tail;
        }

        eprintln!("RESULT_OCAML={:?}", ocaml_hash);

        let bytes = rt.get(bytes);

        let msg: MessagesForNextWrapProof = deserialize(bytes.as_bytes());
        let rust_hash = msg.hash();

        eprintln!("RESULT_RUST_={:?}", rust_hash);

        assert_eq!(&rust_hash[..], ocaml_hash);

        OCaml::unit()
    }

    // fn rust_get_random_message(
    //     rt,
    //     validate_msg: OCamlRef<fn (OCamlBytes) -> ()>,
    // ) -> OCaml<OCamlBytes> {
    //     let mut msg;
    //     let mut bytes;
    //     let validate_msg = validate_msg.to_boxroot(rt);

    //     loop {
    //         msg = MessagesForNextStepProof::rand();
    //         bytes = serialize(&msg);

    //         println!("LOOP");

    //         if validate_msg.try_call(rt, &bytes).is_ok() {
    //             break;
    //         }
    //     }

    //     bytes.to_ocaml(rt)
    // }
}
