use ark_ff::{Field, One};
use ark_poly::Radix2EvaluationDomain;
use kimchi::{curve::KimchiCurve, proof::ProofEvaluations};
use mina_hasher::Fp;
use o1_utils::FieldHelpers;

use super::scalars::{complete_add, endo_mul, endo_mul_scalar, var_base_mul};
use crate::{proofs::public_input::scalar_challenge::endo_fp, util::FpExt};

pub struct PlonkMinimal {
    pub alpha: Fp,
    pub beta: Fp,
    pub gamma: Fp,
    pub zeta: Fp,
    pub joint_combiner: Option<Fp>,
    pub alpha_bytes: [u64; 2],
    pub beta_bytes: [u64; 2],
    pub gamma_bytes: [u64; 2],
    pub zeta_bytes: [u64; 2],
}

type TwoFields = [Fp; 2];

pub struct ScalarsEnv {
    pub zk_polynomial: Fp,
    pub zeta_to_n_minus_1: Fp,
    pub srs_length_log2: u64,
    pub domain: Radix2EvaluationDomain<Fp>,
    pub omega_to_minus_3: Fp,
}

// Result of `plonk_derive`
#[derive(Debug)]
pub struct InCircuit {
    pub alpha: Fp,
    pub beta: Fp,
    pub gamma: Fp,
    pub zeta: Fp,
    pub zeta_to_domain_size: ShiftedValue<Fp>,
    pub zeta_to_srs_length: ShiftedValue<Fp>,
    pub vbmul: ShiftedValue<Fp>,
    pub complete_add: ShiftedValue<Fp>,
    pub endomul: ShiftedValue<Fp>,
    pub endomul_scalar: ShiftedValue<Fp>,
    pub perm: ShiftedValue<Fp>,
}

pub struct Shift<F: Field> {
    c: F,
    scale: F,
}

impl<F> Shift<F>
where
    F: Field + From<i32>,
{
    /// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles_types/shifted_value.ml#L121
    pub fn create() -> Self {
        let c = (0..255).fold(F::one(), |accum, _| accum + accum) + F::one();

        let scale: F = 2.into();
        let scale = scale.inverse().unwrap();

        Self { c, scale } // TODO: This can be a constant
    }
}

pub struct ShiftedValue<F: Field> {
    pub shifted: F,
}

impl<F: Field + FpExt> std::fmt::Debug for ShiftedValue<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShiftedValue")
            .field("shifted", &{
                let mut bytes = self.shifted.to_bytes();
                bytes.reverse();
                hex::encode(bytes)
            })
            .finish()
    }
}

impl<F> ShiftedValue<F>
where
    F: Field,
{
    /// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles_types/shifted_value.ml#L127
    pub fn of_field(field: F, shift: &Shift<F>) -> Self {
        Self {
            shifted: (field - shift.c) * shift.scale,
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles_types/shifted_value.ml#L131
    #[allow(unused)]
    pub fn to_field(&self, shift: &Shift<F>) -> F {
        self.shifted + self.shifted + shift.c
    }
}

/// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L218
pub const PERM_ALPHA0: usize = 21;

pub const NPOWERS_OF_ALPHA: usize = PERM_ALPHA0 + 3;

/// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L141
pub fn powers_of_alpha(alpha: Fp) -> Box<[Fp; NPOWERS_OF_ALPHA]> {
    // The OCaml code computes until alpha^71, but we don't need that much here
    let mut alphas = Box::new([Fp::one(); NPOWERS_OF_ALPHA]);

    alphas[1] = alpha;
    for i in 2..alphas.len() {
        alphas[i] = alpha * alphas[i - 1];
    }

    alphas
}

pub fn derive_plonk(
    env: &ScalarsEnv,
    evals: &ProofEvaluations<TwoFields>,
    minimal: &PlonkMinimal,
) -> InCircuit {
    let shift = Shift::<Fp>::create();

    let zkp = env.zk_polynomial;
    let powers_of_alpha = powers_of_alpha(minimal.alpha);
    let alpha_pow = |i: usize| powers_of_alpha[i];
    let w0 = evals.w.map(|fields| fields[0]);

    let beta = minimal.beta;
    let gamma = minimal.gamma;

    // https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L397
    let perm = evals.s.iter().enumerate().fold(
        evals.z[1] * beta * alpha_pow(PERM_ALPHA0) * zkp,
        |accum, (index, elem)| accum * (gamma + (beta * elem[0]) + w0[index]),
    );
    let perm = -perm;

    // https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L402
    // let generic = {
    //     let [l1, r1, o1, l2, r2, o2, ..] = w0;
    //     let m1 = l1 * r1;
    //     let m2 = l2 * r2;
    //     [evals.generic_selector[0], l1, r1, o1, m1, l2, r2, o2, m2]
    // };

    let zeta_to_domain_size = env.zeta_to_n_minus_1 + Fp::one();
    // https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L46
    let zeta_to_srs_length = (0..env.srs_length_log2).fold(minimal.zeta, |accum, _| accum * accum);

    let complete_add = complete_add(evals, &powers_of_alpha);
    let vbmul = var_base_mul(evals, &powers_of_alpha);
    let endomul = endo_mul(evals, &powers_of_alpha);
    let endomul_scalar = endo_mul_scalar(evals, &powers_of_alpha);

    // Shift values
    let shift = |f| ShiftedValue::of_field(f, &shift);

    InCircuit {
        alpha: minimal.alpha,
        beta: minimal.beta,
        gamma: minimal.gamma,
        zeta: minimal.zeta,
        zeta_to_domain_size: shift(zeta_to_domain_size),
        zeta_to_srs_length: shift(zeta_to_srs_length),
        vbmul: shift(vbmul),
        complete_add: shift(complete_add),
        endomul: shift(endomul),
        endomul_scalar: shift(endomul_scalar),
        perm: shift(perm),
    }
}

pub fn make_shifts(
    domain: &Radix2EvaluationDomain<Fp>,
) -> kimchi::circuits::polynomials::permutation::Shifts<Fp> {
    // let value = 1 << log2_size;
    // let domain = Domain::<Fq>::new(value).unwrap();
    kimchi::circuits::polynomials::permutation::Shifts::new(domain)
}

fn constant_term(
    minimal: &PlonkMinimal,
    env: &ScalarsEnv,
    evals: &ProofEvaluations<[Fp; 2]>,
) -> Fp {
    let constants = kimchi::circuits::expr::Constants {
        alpha: minimal.alpha,
        beta: minimal.beta,
        gamma: minimal.gamma,
        joint_combiner: None,
        endo_coefficient: endo_fp(),
        mds: &mina_curves::pasta::Vesta::sponge_params().mds,
    };

    let evals = evals.map_ref(&|[zeta, zeta_omega]| kimchi::proof::PointEvaluations {
        zeta: *zeta,
        zeta_omega: *zeta_omega,
    });

    let feature_flags = kimchi::circuits::constraints::FeatureFlags {
        range_check0: false,
        range_check1: false,
        foreign_field_add: false,
        foreign_field_mul: false,
        xor: false,
        rot: false,
        lookup_features: kimchi::circuits::lookup::lookups::LookupFeatures {
            patterns: kimchi::circuits::lookup::lookups::LookupPatterns {
                xor: false,
                lookup: false,
                range_check: false,
                foreign_field_mul: false,
            },
            joint_lookup_used: false,
            uses_runtime_tables: false,
        },
    };

    let (linearization, _powers_of_alpha) =
        kimchi::linearization::expr_linearization(Some(&feature_flags), true);

    kimchi::circuits::expr::PolishToken::evaluate(
        &linearization.constant_term,
        env.domain,
        minimal.zeta,
        &evals,
        &constants,
    )
    .unwrap()
}

pub fn ft_eval0(
    env: &ScalarsEnv,
    evals: &ProofEvaluations<[Fp; 2]>,
    minimal: &PlonkMinimal,
    p_eval0: Fp,
) -> Fp {
    const PLONK_TYPES_PERMUTS_MINUS_1_N: usize = 6;

    let e0_s: Vec<_> = evals.s.iter().map(|s| s[0]).collect();
    let zkp = env.zk_polynomial;
    let powers_of_alpha = powers_of_alpha(minimal.alpha);
    let alpha_pow = |i: usize| powers_of_alpha[i];

    let zeta1m1 = env.zeta_to_n_minus_1;
    let w0: Vec<_> = evals.w.iter().map(|w| w[0]).collect();

    let ft_eval0 = {
        let a0 = alpha_pow(PERM_ALPHA0);
        let w_n = w0[PLONK_TYPES_PERMUTS_MINUS_1_N];
        let init = (w_n + minimal.gamma) * evals.z[1] * a0 * zkp;
        e0_s.iter().enumerate().fold(init, |acc, (i, s)| {
            ((minimal.beta * s) + w0[i] + minimal.gamma) * acc
        })
    };

    let shifts = make_shifts(&env.domain);
    let shifts = shifts.shifts();
    let ft_eval0 = ft_eval0 - p_eval0;

    let ft_eval0 = ft_eval0
        - shifts
            .iter()
            .enumerate()
            .fold(alpha_pow(PERM_ALPHA0) * zkp * evals.z[0], |acc, (i, s)| {
                acc * (minimal.gamma + (minimal.beta * minimal.zeta * s) + w0[i])
            });

    let nominator = (zeta1m1 * alpha_pow(PERM_ALPHA0 + 1) * (minimal.zeta - env.omega_to_minus_3)
        + (zeta1m1 * alpha_pow(PERM_ALPHA0 + 2) * (minimal.zeta - Fp::one())))
        * (Fp::one() - evals.z[0]);

    let denominator = (minimal.zeta - env.omega_to_minus_3) * (minimal.zeta - Fp::one());
    let ft_eval0 = ft_eval0 + (nominator / denominator);
    let constant_term = constant_term(minimal, env, evals);

    ft_eval0 - constant_term
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    // #[test]
    // fn test_derive_plonk() {
    //     let f = |s| Fp::from_str(s).unwrap();

    //     let shift = Shift::<Fp>::create();

    //     assert_eq!(
    //         shift.scale.to_decimal(),
    //         "14474011154664524427946373126085988481681528240970780357977338382174983815169"
    //     );
    //     assert_eq!(
    //         shift.c.to_decimal(),
    //         "28948022309329048855892746252171976963271935850878721303774115239606597189632"
    //     );

    //     let env = ScalarsEnv {
    //         zk_polynomial: f(
    //             "14952139847623627632005961777062011953737808795126043460542801649311194043823",
    //         ),
    //         zeta_to_n_minus_1: f(
    //             "19992360331803571005450582443613456929944945612236598221548387723463412653399",
    //         ),
    //         srs_length_log2: 16,
    //     };
    //     let evals = ProofEvaluations {
    //         w: [
    //             [
    //                 f("6289128557598946688693552667439393426405656688717900311656646754749459718720"),
    //                 f("24608814230259595932281883624655184390098983971078865819273833140490521621830")
    //             ],
    //             [
    //                 f("13310290304507948299374922965790520471341407062194282657371827929238607707213"),
    //                 f("3420090439313334404412034509907098823576869881992393349685606692932938435891")
    //             ],
    //             [
    //                 f("1004170231282419143892553232264207846476468710370262861712645500613730936589"),
    //                 f("15358892959463725686065496984540565674655335654104110656651674381972851539319")
    //             ],
    //             [
    //                 f("14235022520036276432047767790226929227376034059765238012530447329202034471184"),
    //                 f("17857867215014615248343791853134492935501050027762182094448303054841389536812")
    //             ],
    //             [
    //                 f("217270972334916519789201407484219134254562938054977741353993356478916733888"),
    //                 f("23568343258930877322113518854070923552760559626728211948238368157503758958395")
    //             ],
    //             [
    //                 f("20724860985472235937562434615223247979133698324833013342416503003953673114269"),
    //                 f("14230270274902068449746409862542021948616341843532980943017138764071240362770")
    //             ],
    //             [
    //                 f("12691818116679147143874935777710847698261799259714916958466576430559681885637"),
    //                 f("22289834256183911112986280165603148282177788815718258800419577474647368509415")
    //             ],
    //             [
    //                 f("24411935077464468269852858502666516692114510902574937532215362000430706390980"),
    //                 f("27001187352494410500619676382138528783481518663974608697374943352528963038629")
    //             ],
    //             [
    //                 f("6360373480154519328489111512851611048061657527472483396338572585132690211046"),
    //                 f("23582754696949565264224763776269416039258289346076169583123362754778051868081")
    //             ],
    //             [
    //                 f("24503282241957787061015546660977633360122189424659418210184685296100040089763"),
    //                 f("1245945804906356625120959596915782469823165927957553537115988479791001371335")
    //             ],
    //             [
    //                 f("9672494201562279236249240210670811621086234303424478665245302624553285994017"),
    //                 f("6320456619637925667340696511492775125069306866183434852057714045546561260366")
    //             ],
    //             [
    //                 f("5210254176721326039284483791950162878174035416676107991148934478856221337173"),
    //                 f("815800705957329676302064392632236617950232059455550504081221846882909342121")
    //             ],
    //             [
    //                 f("19271726941980627641895844001132986442331047173347430774843986312900315111686"),
    //                 f("20110056132616893796657499795094959354081479634362692363043782275259311243305")
    //             ],
    //             [
    //                 f("146495953729640570485828778213514297261651081782921764417219235177307212109"),
    //                 f("13510051022748561152196281174320974475938901566632935888293972337640213044221")
    //             ],
    //             [
    //                 f("3518198406204532554527858244346702174770464435733756876385351085395310209176"),
    //                 f("15550522660048386236921180860694193742171605550559912660860741398328060104279")
    //             ],
    //         ],
    //         z: [
    //             f("21931198523501459183153033195666929989603803804227249746439702940933841410354"),
    //             f("6022590485205496790548288878896488149850531459551034305056017859745758834601")
    //         ],
    //         s: [
    //             [
    //                 f("19085168690544843887043640753077042378743448700110307869084708762711870533936"),
    //                 f("22751786545661378548233843701817138109179015540519447359893734727224759557557")
    //             ],
    //             [
    //                 f("12809849114708285788093252766158738590831386282299128882364744994146092851397"),
    //                 f("7775219585837024399019590773584893518283271570804407455680225353290518221037")
    //             ],
    //             [
    //                 f("1321451461746223492813831865923699791759758119520679872448309408329685991365"),
    //                 f("24250442489736828467922579478325926748516761175781497417627070248911524273876")
    //             ],
    //             [
    //                 f("14126289132628522291939529284539290777701180720493062389536649540269039839059"),
    //                 f("9881670171615426333925133765399382666579015176251728331663175340398678714235")
    //             ],
    //             [
    //                 f("5478696824111960874152427151223870065214252944402491332456328657031528756061"),
    //                 f("1377997571297099342686120389926615964800209763625383648915112029912281716920")
    //             ],
    //             [
    //                 f("80056797119034209825231055487115732036243341031547669733590224790110901245"),
    //                 f("23954132758389233312829853625715975217840278999681825436710260294777706132878")
    //             ],
    //         ],
    //         generic_selector: [
    //             f("21284673669227882794919992407172292293537574509733523059283928016664652610788"),
    //             f("21551467950274947783566469754812261989391861302105687694024368348901050138933")
    //         ],
    //         poseidon_selector: [
    //             f("24296813547347387275700638062514083772319473069038997762814161750820112396767"),
    //             f("7410307958356630828270235926030893251108745125559587079838321820967330039556")
    //         ],
    //         lookup: None,
    //     };

    //     let minimal = PlonkMinimal {
    //         alpha: f(
    //             "27274897876953793245985525242013286410205575357216365244783619058623821516088",
    //         ),
    //         beta: f("325777784653629264882054754458727445360"),
    //         gamma: f("279124633756639571190538225494418257063"),
    //         zeta: f(
    //             "23925312945193845374476523404515906217333838946294378278132006013444902630130",
    //         ),
    //         joint_combiner: None,
    //         alpha_bytes: [0; 2], // unused here
    //         beta_bytes: [0; 2],  // unused here
    //         gamma_bytes: [0; 2], // unused here
    //         zeta_bytes: [0; 2],  // unused here
    //     };

    //     let plonk = derive_plonk(&env, &evals, &minimal);

    //     let InCircuit {
    //         alpha,
    //         beta,
    //         gamma,
    //         zeta,
    //         zeta_to_domain_size,
    //         zeta_to_srs_length,
    //         poseidon_selector,
    //         vbmul,
    //         complete_add,
    //         endomul,
    //         endomul_scalar,
    //         perm,
    //         generic,
    //     } = plonk;

    //     // OCAML RESULTS
    //     assert_eq!(
    //         alpha.to_decimal(),
    //         "27274897876953793245985525242013286410205575357216365244783619058623821516088"
    //     );
    //     assert_eq!(beta.to_decimal(), "325777784653629264882054754458727445360");
    //     assert_eq!(
    //         gamma.to_decimal(),
    //         "279124633756639571190538225494418257063"
    //     );
    //     assert_eq!(
    //         zeta.to_decimal(),
    //         "23925312945193845374476523404515906217333838946294378278132006013444902630130"
    //     );
    //     assert_eq!(
    //         zeta_to_srs_length.shifted.to_decimal(),
    //         "24470191320566309930671664347892716946699561362620499174841813006278375362221"
    //     );
    //     assert_eq!(
    //         zeta_to_domain_size.shifted.to_decimal(),
    //         "24470191320566309930671664347892716946699561362620499174841813006278375362221"
    //     );
    //     assert_eq!(
    //         poseidon_selector.shifted.to_decimal(),
    //         "12148406773673693637850319031257041886205296850050918587497361637781741418736"
    //     );
    //     assert_eq!(
    //         perm.shifted.to_decimal(),
    //         "23996362553795482752052846938731572092036494641475074785875316421561912520951"
    //     );

    //     assert_eq!(
    //         complete_add.shifted.to_decimal(),
    //         "25922772146036107832349768103654536495710983785678446578187694836830607595414",
    //     );
    //     assert_eq!(
    //         vbmul.shifted.to_decimal(),
    //         "28215784507806213626095422113975086465418154941914519667351031525311316574704",
    //     );
    //     assert_eq!(
    //         endomul.shifted.to_decimal(),
    //         "13064695703283280169401378869933492390852015768221327952370116298712909203140",
    //     );
    //     assert_eq!(
    //         endomul_scalar.shifted.to_decimal(),
    //         "28322098634896565337442184680409326841751743190638136318667225694677236113253",
    //     );

    //     let generic_str = generic.map(|f| f.shifted.to_decimal());
    //     assert_eq!(
    //         generic_str,
    //         [
    //             "25116347989278465825406369329672134628495875811368961593709583152878995340915",
    //             "17618575433463997772293149459805685194929916900861150219895942521921398894881",
    //             "6655145152253974149687461482895260235716263846628561034776194726990989073959",
    //             "502085115641209571946276616132103923283794670716551136946603512678550688647",
    //             "7575395148088980464707243648576550930395639510870089332970629398518203372685",
    //             "21591522414682662643970257021199453095415105586384819070332842809147686271113",
    //             "14582646640831982687840973829828098048854370025529688934744615822786127402465",
    //             "10362430492736117968781217307611623989612409477947926377298532264348521777487",
    //             "12100020790372419869711605834757334981877832164856370998628117306965052030192",
    //         ]
    //     );
    // }

    #[test]
    fn test_alphas() {
        let n = Fp::from_str(
            "27274897876953793245985525242013286410205575357216365244783619058623821516088",
        )
        .unwrap();

        let alphas: Box<[Fp; NPOWERS_OF_ALPHA]> = powers_of_alpha(n);
        let alphas_str: Vec<String> = alphas.iter().map(|f| f.to_decimal()).collect();

        const OCAML_RESULT: &[&str] = &[
            "1",
            "27274897876953793245985525242013286410205575357216365244783619058623821516088",
            "5856243499679297994261942705106783326584825647279332525318074626467168425175",
            "26908526253468636093650206549302380737071523922183255477383956748755441012366",
            "21276200075660690362913766498168565850417909161152737384646582509540496229450",
            "3843731251681147173193384676587074004662025496739119332721571982426684387560",
            "12392606098341916760701161625583524765199435768082801118718099569066567820086",
            "5932489972119399045562481763112253944218195162891420406370178296693572483896",
            "1375846522483390900802414356841133463956126287864007136159978297384640659584",
            "5356524575738460513076981906288272723856440519543693179836517878630162813220",
            "23319398249603527452857836680743813857193813763032214190196631633251915644825",
            "10921184148344839491052929288136821627436657352065581423854521247501001908351",
            "13053560967285308651226207033123539702290413328361716005386653453569329750313",
            "8298101552564684053050013414211292674866114224797784754887740268228151928335",
            "715072795965317694491886715913315968459520650830405802156784401283709943505",
            "25198551493059869063561311792478884528738012039746184861146867788131566740666",
            "27161703551928606962685117055547438689494792119791879693135179256422752270728",
            "28799358614011589987311924793640447939591189984280570017428244220659375622447",
            "4488279652568453906961591843014473441709515392753701104095475657832824041646",
            "4641946865609115816676535679719511429699894348223929677606063307711524129548",
            "995093492640264169583875280706844374785298168266651011740457078469635678163",
            "17429526728376789811772110265115435172515921536052154380599101096979177652072",
        ];

        assert_eq!(alphas_str, OCAML_RESULT);
    }
}
