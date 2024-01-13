use std::array::IntoIter;

use ark_ff::{BigInteger256, Field};

use crate::proofs::witness::FieldWitness;

#[derive(Clone, Debug)]
pub struct ScalarChallenge {
    pub inner: [u64; 2],
}

impl<F: FieldWitness> From<F> for ScalarChallenge {
    fn from(value: F) -> Self {
        let bigint: BigInteger256 = value.into();
        Self::new(bigint.0[0], bigint.0[1])
    }
}

impl From<[u64; 2]> for ScalarChallenge {
    fn from(value: [u64; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<Vec<u64>> for ScalarChallenge {
    fn from(value: Vec<u64>) -> Self {
        Self::new(value[0], value[1])
    }
}

struct ScalarChallengeBitsIterator {
    inner: IntoIter<bool, 128>,
}

impl Iterator for ScalarChallengeBitsIterator {
    type Item = (bool, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let second = self.inner.next()?;
        let first = self.inner.next()?;
        Some((first, second))
    }
}

impl ScalarChallenge {
    pub fn new(a: u64, b: u64) -> Self {
        Self { inner: [a, b] }
    }

    pub fn dummy() -> Self {
        Self::new(1, 1)
    }

    fn iter_bits(&self) -> ScalarChallengeBitsIterator {
        let a: u128 = self.inner[0] as u128;
        let b: u128 = self.inner[1] as u128;
        let num: u128 = (a | (b << 64)).reverse_bits();

        let mut bits = [false; 128];
        for (index, bit) in bits.iter_mut().enumerate() {
            *bit = ((num >> index) & 1) != 0;
        }

        ScalarChallengeBitsIterator {
            inner: bits.into_iter(),
        }
    }

    /// Implemention of `to_field_constant`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/scalar_challenge.ml#L139
    pub fn to_field<F>(&self, endo: &F) -> F
    where
        F: Field + From<i32>,
    {
        let mut a: F = 2.into();
        let mut b: F = 2.into();
        let one: F = 1.into();
        let neg_one: F = -one;

        for (first, second) in self.iter_bits() {
            let s = if first { one } else { neg_one };

            a += a;
            b += b;

            if second {
                a += s;
            } else {
                b += s;
            }
        }

        (a * endo) + b
    }
}

#[cfg(test)]
mod tests {
    use crate::util::FpExt;

    use super::*;

    use mina_curves::pasta::Fq;
    use mina_hasher::Fp;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn test_fp_challenges() {
        #[rustfmt::skip]
        let scalar_challenges = [
            ScalarChallenge::new(-6073566958339139610i64 as u64, -2081966045668129095i64 as u64),
            ScalarChallenge::new(-7959424372843801919i64 as u64,4324727692349798413i64 as u64),
            ScalarChallenge::new(5281407247363008654i64 as u64,-341126770839524452i64 as u64),
            ScalarChallenge::new(-8793505104561934341i64 as u64,1149061412269814663i64 as u64),
            ScalarChallenge::new(-6319682980234921909i64 as u64,-2993470668064492538i64 as u64),
            ScalarChallenge::new(-4632520420895617322i64 as u64,-1043277042477928353i64 as u64),
            ScalarChallenge::new(-6520172058534332359i64 as u64,-6056327257651418415i64 as u64),
            ScalarChallenge::new(2243596438622148941i64 as u64,2765464070332650197i64 as u64),
            ScalarChallenge::new(2003371658010473514i64 as u64,-4740690826649485901i64 as u64),
            ScalarChallenge::new(4360179541971010175i64 as u64,-1774460476906845353i64 as u64),
            ScalarChallenge::new(1218693385180536706i64 as u64,7371465240007863116i64 as u64),
            ScalarChallenge::new(-4074108557518982675i64 as u64,-1941843029663935209i64 as u64),
            ScalarChallenge::new(-443524479034963553i64 as u64,-6709701916849696870i64 as u64),
            ScalarChallenge::new(-8471733636946508646i64 as u64,-7855667751781032417i64 as u64),
            ScalarChallenge::new(-4458439752495496649i64 as u64,5085686095541820097i64 as u64),
            ScalarChallenge::new(7895667244538374865i64 as u64,-7599583809327744882i64 as u64),
        ];

        let (_, endo) = crate::proofs::witness::endos::<Fq>();

        let challenges: Vec<_> = scalar_challenges
            .iter()
            .rev()
            .map(|s| s.to_field(&endo).to_decimal())
            .collect();

        const OCAML_RESULTS: &[&str] = &[
            "7064008822864728398858417698649153554618393735019982159700213736571223036821",
            "21281073930096202884976340572605376014917627927607000098124963387126671667405",
            "20034221077127088921131022285724504665904750595809121027858777196941115095595",
            "18497339512896533967070595151637618702096421683793996188803218728257093313954",
            "26091707259541979535386026279646546924481219402460945255985587200333895442426",
            "26000802478877378291818950871303958371690312575800175603275158984631747112314",
            "3542775955255721718735057551619208290322691095579511881465771193916627523656",
            "26082193213255146858325625064941365409422644280364303832412501842706109482352",
            "14045706980484410694469015017617110793948725477238650483822693423475791735584",
            "19471582594431150457263082825199126200392919902939760347730982393598455884567",
            "1326396224493213224809637465524487722432218763470273737330947122279510339127",
            "17485742224041066720858261709734580297392897788368828400728418930917070940077",
            "11369682275649236437578679786208728227049581168916414391705587421204991504471",
            "5558430160842950378525019782179979114837605858253956740879954326712738838869",
            "17514922848108718369845316737207968652832912751889238861905018424256428652733",
            "11088960946452242729814251490831984807138805895197664788816609458265399565988",
        ];

        assert_eq!(challenges, OCAML_RESULTS);
    }

    #[test]
    fn test_fq_challenges() {
        #[rustfmt::skip]
        let scalar_challenges = [
            ScalarChallenge::new(7486980280913238963i64 as u64,4173194488927267133i64 as u64),
            ScalarChallenge::new(-8437921285878338178i64 as u64,-2241273202573544127i64 as u64),
            ScalarChallenge::new(7651331705457292674i64 as u64,-3583141513394030281i64 as u64),
            ScalarChallenge::new(-3464302417307075879i64 as u64,-436261906098457727i64 as u64),
            ScalarChallenge::new(8255044994932440761i64 as u64,5640094314955753085i64 as u64),
            ScalarChallenge::new(-2513734760972484960i64 as u64,1161566061253204655i64 as u64),
            ScalarChallenge::new(7525998242613288472i64 as u64,3436443803216159028i64 as u64),
            ScalarChallenge::new(6809231383204761158i64 as u64,-1877195934091894696i64 as u64),
            ScalarChallenge::new(-2746520749286704399i64 as u64,-3783224604272248786i64 as u64),
            ScalarChallenge::new(-36686536733916892i64 as u64,-7835584350097226223i64 as u64),
            ScalarChallenge::new(-487486487490201322i64 as u64,2756145684490201109i64 as u64),
            ScalarChallenge::new(-2928903316653004982i64 as u64,346819656816504982i64 as u64),
            ScalarChallenge::new(-6510054999844554738i64 as u64,5242613218253829938i64 as u64),
            ScalarChallenge::new(-9192160905410203809i64 as u64,9069127704639200224i64 as u64),
            ScalarChallenge::new(-1805085648820294365i64 as u64,4705625510417283644i64 as u64),
        ];

        let (_, endo) = crate::proofs::witness::endos::<Fp>();

        let challenges: Vec<_> = scalar_challenges
            .iter()
            .map(|s| s.to_field(&endo).to_decimal())
            .collect();

        const OCAML_RESULTS: &[&str] = &[
            "18930573265662216159442494814184247548231007813972071269603851935535469568681",
            "13262618632343563854609319990042165233884668386778849308201250967495466297892",
            "3280133380740312520798484208652485449441154026649408650609268816997503996596",
            "5974304366701279384657274575561097681967365179758505920855016184604375547845",
            "3661423464518527108104215048163162714022670481055215775284279432335542360947",
            "28186039308015693212526267835091480366420716733534501029059512929488448172803",
            "2928242866467619037394221722592602237395615492823915494722307192257841920053",
            "7996242790950647686981276345325562931441969146107262270102737887450525547679",
            "21300612581975001967226090647975097001166598554510871181982133469971233105666",
            "20608125931317303080428446946426401578478104508733856541747380631345253223759",
            "20544136014497620836884335380316800592936676638960436539103183677666423950244",
            "18427567011408397204090055018472561464711254957456387310196545464234121256500",
            "17571157148266047799767015777008435938200309464425419775013145733714886071865",
            "9174651000781852149559362347943707955252174789399970369305126738373271491014",
            "2598585929268909637848366187420073930931414900720607825745008437243507445619",
        ];

        assert_eq!(challenges, OCAML_RESULTS);
    }
}
