use super::{
    constants::ProofConstants,
    field::{FieldWitness, GroupAffine},
    to_field_elements::ToFieldElements,
    transaction::{add_fast, scalar_challenge, Check},
};

#[derive(Debug)]
pub struct Witness<F: FieldWitness> {
    pub primary: Vec<F>,
    pub(super) aux: Vec<F>,
    // Following fields are used to compare our witness with OCaml
    pub ocaml_aux: Vec<F>,
    ocaml_aux_index: usize,
}

impl<F: FieldWitness> Witness<F> {
    pub fn new<C: ProofConstants>() -> Self {
        Self {
            primary: Vec::with_capacity(C::PRIMARY_LEN),
            aux: Vec::with_capacity(C::AUX_LEN),
            ocaml_aux: Vec::new(),
            ocaml_aux_index: 0,
        }
    }

    pub fn empty() -> Self {
        Self {
            primary: Vec::new(),
            aux: Vec::new(),
            ocaml_aux: Vec::new(),
            ocaml_aux_index: 0,
        }
    }

    pub fn push<I: Into<F>>(&mut self, field: I) {
        let field = {
            let field: F = field.into();
            // dbg!(field)
            field
        };
        self.assert_ocaml_aux(&[field]);
        self.aux.push(field);
    }

    pub fn extend<I: Into<F>, V: Iterator<Item = I>>(&mut self, field: V) {
        let fields = {
            let fields: Vec<F> = field.map(Into::into).collect();
            self.assert_ocaml_aux(&fields);
            // eprintln!("extend[{}]={:#?}", fields.len(), fields);
            fields
        };
        self.aux.extend(fields)
    }

    pub fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>,
    {
        // data.to_field_elements(&mut self.aux);
        let mut fields = data.to_field_elements_owned();
        self.assert_ocaml_aux(&fields);

        // eprintln!("index={:?} w{:?}", self.aux.len() + 67, &fields);
        if self.ocaml_aux.len() > 0 {
            eprintln!(
                "index={:?} w{:?}",
                self.aux.len() + self.primary.capacity(),
                &fields
            );
        }
        self.aux.append(&mut fields);

        data.check(self);
        data
    }

    pub fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>,
    {
        // data.to_field_elements(&mut self.aux);
        let mut fields = data.to_field_elements_owned();
        self.assert_ocaml_aux(&fields);

        // eprintln!("index={:?} w{:?}", self.aux.len() + 67, &fields);
        if self.ocaml_aux.len() > 0 {
            eprintln!(
                "index={:?} w{:?}",
                self.aux.len() + self.primary.capacity(),
                &fields
            );
        }
        self.aux.append(&mut fields);

        data
    }

    /// Compare our witness with OCaml
    fn assert_ocaml_aux(&mut self, new_fields: &[F]) {
        if self.ocaml_aux.is_empty() {
            return;
        }

        // let len = new_fields.len();
        // let before = self.aux.len();
        // let ocaml = &self.ocaml_aux[before..before + len];
        // eprintln!("w{:?} ocaml{:?} {:?}", new_fields, ocaml, new_fields == ocaml);

        let len = new_fields.len();
        let before = self.aux.len();
        assert_eq!(before, self.ocaml_aux_index);
        assert_eq!(new_fields, &self.ocaml_aux[before..before + len]);

        self.ocaml_aux_index += len;
    }

    /// Helper
    pub fn to_field_checked_prime<const NBITS: usize>(&mut self, scalar: F) -> (F, F, F) {
        scalar_challenge::to_field_checked_prime::<F, NBITS>(scalar, self)
    }

    /// Helper
    pub fn add_fast(&mut self, p1: GroupAffine<F>, p2: GroupAffine<F>) -> GroupAffine<F> {
        add_fast::<F>(p1, p2, None, self)
    }
}
