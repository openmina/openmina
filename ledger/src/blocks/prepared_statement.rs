use std::str::FromStr;

use ark_ff::{BigInteger256, PrimeField};
use bitvec::macros::internal::funty::Fundamental;
use mina_curves::pasta::Fq;

use crate::ProofVerified;

type ShiftedValue = String;

struct Plonk {
    alpha: [u64; 2],
    beta: [u64; 2],
    gamma: [u64; 2],
    zeta: [u64; 2],
    zeta_to_srs_length: ShiftedValue,
    zeta_to_domain_size: ShiftedValue,
    poseidon_selector: ShiftedValue,
    vbmul: ShiftedValue,
    complete_add: ShiftedValue,
    endomul: ShiftedValue,
    endomul_scalar: ShiftedValue,
    perm: ShiftedValue,
    generic: Vec<ShiftedValue>,
    lookup: (),
}

struct BranchData {
    proofs_verified: ProofVerified,
    domain_log2: char,
}

struct DeferredValues {
    plonk: Plonk,
    combined_inner_product: ShiftedValue,
    b: ShiftedValue,
    xi: [u64; 2],
    bulletproof_challenges: Vec<[u64; 2]>,
    branch_data: BranchData,
}

struct ProofState {
    deferred_values: DeferredValues,
    sponge_digest_before_evaluations: [u64; 4],
    messages_for_next_wrap_proof: [u64; 4],
}

struct PreparedStatement {
    proof_state: ProofState,
    messages_for_next_step_proof: [u64; 4],
}

fn two_u64_to_field(v: &[u64; 2]) -> Fq {
    let mut bigint: [u64; 4] = [0; 4];
    bigint[..2].copy_from_slice(v);

    let bigint = BigInteger256(bigint);
    Fq::from_repr(bigint).unwrap()
}

fn four_u64_to_field(v: &[u64; 4]) -> Fq {
    let mut bigint: [u64; 4] = [0; 4];
    bigint.copy_from_slice(v);

    let bigint = BigInteger256(bigint);
    Fq::from_repr(bigint).unwrap()
}

impl PreparedStatement {
    /// Implementation of `tock_unpadded_public_input_of_statement`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/common.ml#L202
    pub fn to_fields(&self) -> Vec<Fq> {
        let PreparedStatement {
            proof_state:
                ProofState {
                    deferred_values:
                        DeferredValues {
                            plonk:
                                Plonk {
                                    alpha,
                                    beta,
                                    gamma,
                                    zeta,
                                    zeta_to_srs_length,
                                    zeta_to_domain_size,
                                    poseidon_selector,
                                    vbmul,
                                    complete_add,
                                    endomul,
                                    endomul_scalar,
                                    perm,
                                    generic,
                                    lookup,
                                },
                            combined_inner_product,
                            b,
                            xi,
                            bulletproof_challenges,
                            branch_data:
                                BranchData {
                                    proofs_verified,
                                    domain_log2,
                                },
                        },
                    sponge_digest_before_evaluations,
                    messages_for_next_wrap_proof,
                },
            messages_for_next_step_proof,
        } = &self;

        let f = |s| Fq::from_str(s).unwrap();

        let mut fields = Vec::with_capacity(47);

        fields.push(f(combined_inner_product));
        fields.push(f(b));
        fields.push(f(zeta_to_srs_length));
        fields.push(f(zeta_to_domain_size));
        fields.push(f(poseidon_selector));
        fields.push(f(vbmul));
        fields.push(f(complete_add));
        fields.push(f(endomul));
        fields.push(f(endomul_scalar));
        fields.push(f(perm));

        let generics: Vec<_> = generic.iter().map(|g| f(g)).collect();
        fields.extend_from_slice(&generics);

        fields.push(two_u64_to_field(beta));
        fields.push(two_u64_to_field(gamma));
        fields.push(two_u64_to_field(alpha));
        fields.push(two_u64_to_field(zeta));
        fields.push(two_u64_to_field(xi));

        fields.push(four_u64_to_field(sponge_digest_before_evaluations));
        fields.push(four_u64_to_field(messages_for_next_wrap_proof));
        fields.push(four_u64_to_field(messages_for_next_step_proof));

        for challenge in bulletproof_challenges {
            fields.push(two_u64_to_field(challenge));
        }

        // https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles_base/proofs_verified.ml#L58
        let proofs_verified = match proofs_verified {
            ProofVerified::N0 => 0b00,
            ProofVerified::N1 => 0b10,
            ProofVerified::N2 => 0b11,
        };
        let domain_log2 = domain_log2.as_u64();
        let num: u64 = (domain_log2 << 2) | proofs_verified;

        fields.push(two_u64_to_field(&[num, 0]));

        // TODO: Not sure how that padding works, check further
        fields.push(0.into());
        fields.push(0.into());
        fields.push(0.into());

        assert_eq!(fields.len(), 47);

        fields
    }
}

#[cfg(test)]
mod tests {
    use crate::FpExt;

    use super::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    /// This test the same input/output as
    /// https://gist.githubusercontent.com/tizoc/663b0eed950f21a386da2ae5f1a7b3cf/raw/e885845c800eb43a81c224587deaad6234ad9d30/output.txt
    #[test]
    fn test_prepared_statement() {
        let u = |s| u64::from_str_radix(s, 16).unwrap();
        let s = |s: &str| s.to_string();

        let prepared_statement = PreparedStatement {
            proof_state: ProofState {
                deferred_values: DeferredValues {
                    plonk: Plonk {
                        alpha: [u("92865f85fcb91534"), u("55c34b9548fc683a")],
                        beta: [u("eadeaf2e86d97370"), u("f51684e1a4e564e4")],
                        gamma: [u("c6354f6c55f3f4a7"), u("d1fd72f41370a073")],
                        zeta: [u("c16a9c01c7403f99"), u("f61c19bf39e4f217")],
                        zeta_to_srs_length: s("24470191320566309930671664347892716946699561362620499174841813006278375362221"),
                        zeta_to_domain_size: s("24470191320566309930671664347892716946699561362620499174841813006278375362221"),
                        poseidon_selector: s("12148406773673693637850319031257041886205296850050918587497361637781741418736"),
                        vbmul: s("28215784507806213626095422113975086465418154941914519667351031525311316574704"),
                        complete_add: s("25922772146036107832349768103654536495710983785678446578187694836830607595414"),
                        endomul: s("13064695703283280169401378869933492390852015768221327952370116298712909203140"),
                        endomul_scalar: s("28322098634896565337442184680409326841751743190638136318667225694677236113253"),
                        perm: s("23996362553795482752052846938731572092036494641475074785875316421561912520951"),
                        generic: vec![
                            s("25116347989278465825406369329672134628495875811368961593709583152878995340915"),
                            s("17618575433463997772293149459805685194929916900861150219895942521921398894881"),
                            s("6655145152253974149687461482895260235716263846628561034776194726990989073959"),
                            s("502085115641209571946276616132103923283794670716551136946603512678550688647"),
                            s("7575395148088980464707243648576550930395639510870089332970629398518203372685"),
                            s("21591522414682662643970257021199453095415105586384819070332842809147686271113"),
                            s("14582646640831982687840973829828098048854370025529688934744615822786127402465"),
                            s("10362430492736117968781217307611623989612409477947926377298532264348521777487"),
                            s("12100020790372419869711605834757334981877832164856370998628117306965052030192"),
                        ],
                        lookup: ()
                    },
                    combined_inner_product: s("19748141906434917250055722022154146473680473908068340804972023794620180207895"),
                    b: s("15255844528889452990087363854008126833830217356134354347820655170692488422596"),
                    xi: [u("02d75d3de6e556f2"), u("e87cc65c1624a3ee")],
                    bulletproof_challenges: vec![
                        [u("af2f4fe0711b77f6"), u("5b1b2ef34739a766")],
                        [u("ef1575da47233e48"), u("6221d8f7b2ce3a23")],
                        [u("ee53e229fbf1fb14"), u("9f81dc68f5cf969e")],
                        [u("57c6c59dfcf68878"), u("46d05899284ea3e1")],
                        [u("e6a579e41062b48c"), u("f08bc54a500b541d")],
                        [u("d6d6723f9fb5c00a"), u("bb16daafa0337aa4")],
                        [u("f8ea1a43cc88bcc8"), u("3719da67c309ded1")],
                        [u("58eabeceed0a85ab"), u("fb304f30a011d844")],
                        [u("5a9f772f919994ba"), u("143e015fa02a8710")],
                        [u("0408739adb5d8f4f"), u("52c58b1343c68b9f")],
                        [u("03ff54e1d66986e8"), u("156905f2df1a653f")],
                        [u("0fb2846032910069"), u("8b402b8f7ad03114")],
                        [u("8ea1ec41386d1daf"), u("6442939f89f90e2c")],
                        [u("c79a0142b3804c5e"), u("966f297db3f3c196")],
                        [u("c5af2ef7bc6b8391"), u("fc2c95291c305f16")],
                        [u("a53c284610fb6b2c"), u("d1ee01c37ed8fa96")],
                    ],
                    branch_data: BranchData {
                        proofs_verified: ProofVerified::N2,
                        domain_log2: char::from_u32(16).unwrap(),
                    },
                },
                sponge_digest_before_evaluations: [
                    u("7fd734e499c6a344"),
                    u("3a2643142b053338"),
                    u("5a6e1b49ea9033ba"),
                    u("1a659f082a78f930"),
                ],
                messages_for_next_wrap_proof: [
                    -2757079307213834418i64 as u64,
                    -7374019804543813095i64 as u64,
                    -4454791831784549861i64 as u64,
                    2589444293822685710,
                ],
            },
            messages_for_next_step_proof: [
                7912308706379928291,
                8689988569980666660,
                5997160798854948936,
                3770142804027174900,
            ],
        };

        let fields = prepared_statement.to_fields();
        let fields_str: Vec<_> = fields.iter().map(|f| f.to_decimal()).collect();

        const OCAML_RESULT: &[&str] = &[
            "19748141906434917250055722022154146473680473908068340804972023794620180207895",
            "15255844528889452990087363854008126833830217356134354347820655170692488422596",
            "24470191320566309930671664347892716946699561362620499174841813006278375362221",
            "24470191320566309930671664347892716946699561362620499174841813006278375362221",
            "12148406773673693637850319031257041886205296850050918587497361637781741418736",
            "28215784507806213626095422113975086465418154941914519667351031525311316574704",
            "25922772146036107832349768103654536495710983785678446578187694836830607595414",
            "13064695703283280169401378869933492390852015768221327952370116298712909203140",
            "28322098634896565337442184680409326841751743190638136318667225694677236113253",
            "23996362553795482752052846938731572092036494641475074785875316421561912520951",
            "25116347989278465825406369329672134628495875811368961593709583152878995340915",
            "17618575433463997772293149459805685194929916900861150219895942521921398894881",
            "6655145152253974149687461482895260235716263846628561034776194726990989073959",
            "502085115641209571946276616132103923283794670716551136946603512678550688647",
            "7575395148088980464707243648576550930395639510870089332970629398518203372685",
            "21591522414682662643970257021199453095415105586384819070332842809147686271113",
            "14582646640831982687840973829828098048854370025529688934744615822786127402465",
            "10362430492736117968781217307611623989612409477947926377298532264348521777487",
            "12100020790372419869711605834757334981877832164856370998628117306965052030192",
            "325777784653629264882054754458727445360",
            "279124633756639571190538225494418257063",
            "113998410537436691305318482013880653108",
            "327135993485864835321774365821269720985",
            "309028763045504241164526269977455843058",
            "11939683214529137769731943201373005357591938267283580253152769217863256286020",
            "16254205270441518554807503926541413290547235828119296718777327786019265738574",
            "23665569937814586253210412514094683496449316555579062631605678244743167972067",
            "121100891896936178721717767288613730294",
            "130440090008421579667577446890161258056",
            "212021528070469107744391217378566994708",
            "94127754437947811788804624796909734008",
            "319740449774064592171997262614794253452",
            "248684301222468906458924272218317111306",
            "73241776975789588220005549234348145864",
            "333887063354073219593971432040518288811",
            "26906510179581533554819714345604846778",
            "110022398916739980286721881731671887695",
            "28459099735939804325215087146501244648",
            "185095881924298591398191234097320362089",
            "133268485325344998287119288492740124079",
            "199961385857041310734180261999992130654",
            "335196941335683997330373792655410430865",
            "279044453542537422930767191238321597228",
            "67",
            "0",
            "0",
            "0",
        ];

        assert_eq!(fields_str, OCAML_RESULT);
    }
}
