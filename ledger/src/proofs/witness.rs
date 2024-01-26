use super::{
    constants::ProofConstants,
    field::{FieldWitness, GroupAffine},
    to_field_elements::ToFieldElements,
    transaction::{add_fast, scalar_challenge, Check},
};

#[derive(Debug)]
pub struct Witness<F: FieldWitness> {
    pub(super) primary: Vec<F>,
    aux: Vec<F>,
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

    pub(super) fn aux(&self) -> &[F] {
        &self.aux
    }

    pub fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>,
    {
        let data = self.exists_no_check(data);
        data.check(self);
        data
    }

    /// Same as `Self::exists`, but do not call `Check::check` on `data`
    /// We use this wherever `seal` is used in OCaml, or on `if_` conditions
    pub fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>,
    {
        #[cfg(test)]
        let start = self.aux.len();

        data.to_field_elements(&mut self.aux);

        #[cfg(test)]
        self.assert_ocaml_aux(start);

        data
    }

    /// Compare our witness with OCaml
    #[cfg(test)]
    fn assert_ocaml_aux(&mut self, start_offset: usize) {
        if self.ocaml_aux.is_empty() {
            return;
        }

        let new_fields = &self.aux[start_offset..];
        let len = new_fields.len();
        let ocaml_fields = &self.ocaml_aux[start_offset..start_offset + len];

        assert_eq!(start_offset, self.ocaml_aux_index);
        assert_eq!(new_fields, ocaml_fields);

        self.ocaml_aux_index += len;

        eprintln!(
            "index={:?} w{:?}",
            self.aux.len() + self.primary.capacity(),
            &self.aux[start_offset..]
        );
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
