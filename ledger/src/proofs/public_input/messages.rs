use std::str::FromStr;

use crate::proofs::to_field_elements::ToFieldElements;
use crate::proofs::VerifierIndex;
use crate::{CurveAffine, PlonkVerificationKeyEvals};
use ark_ff::{BigInteger256, PrimeField};
use mina_curves::{pasta::Fq, pasta::Pallas};
use mina_hasher::Fp;
use poly_commitment::PolyComm;

use crate::hash::hash_fields;

impl<'a> From<&'a VerifierIndex> for PlonkVerificationKeyEvals {
    fn from(verifier_index: &'a VerifierIndex) -> Self {
        let to_curve = |v: &PolyComm<Pallas>| {
            let v = v.unshifted[0];
            CurveAffine(v.x, v.y)
        };

        Self {
            sigma: std::array::from_fn(|i| to_curve(&verifier_index.sigma_comm[i])),
            coefficients: std::array::from_fn(|i| to_curve(&verifier_index.coefficients_comm[i])),
            generic: to_curve(&verifier_index.generic_comm),
            psm: to_curve(&verifier_index.psm_comm),
            complete_add: to_curve(&verifier_index.complete_add_comm),
            mul: to_curve(&verifier_index.mul_comm),
            emul: to_curve(&verifier_index.emul_comm),
            endomul_scalar: to_curve(&verifier_index.endomul_scalar_comm),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MessagesForNextWrapProof {
    pub challenge_polynomial_commitment: CurveAffine<Fq>,
    pub old_bulletproof_challenges: Vec<[Fq; 15]>,
}

impl MessagesForNextWrapProof {
    /// Implementation of `hash_messages_for_next_wrap_proof`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/wrap_hack.ml#L50
    pub fn hash(&self) -> [u64; 4] {
        let fields: Vec<Fq> = self.to_fields();
        let field: Fq = hash_fields(&fields);

        let bigint: BigInteger256 = field.into_repr();
        bigint.0
    }

    /// Implementation of `to_field_elements`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/composition_types/composition_types.ml#L356
    fn to_fields(&self) -> Vec<Fq> {
        const NFIELDS: usize = 32;

        let mut fields = Vec::with_capacity(NFIELDS);

        let padding = 2usize
            .checked_sub(self.old_bulletproof_challenges.len())
            .expect("old_bulletproof_challenges must be of length <= 2");

        // TODO: Currently `Self::old_bulletproof_challenges` is always of length 2
        for _ in 0..padding {
            fields.extend_from_slice(&Self::dummy_padding());
        }

        for challenges in &self.old_bulletproof_challenges {
            fields.extend_from_slice(challenges);
        }

        fields.push(self.challenge_polynomial_commitment.0);
        fields.push(self.challenge_polynomial_commitment.1);

        assert_eq!(fields.len(), NFIELDS);

        fields
    }

    /// Value of `Dummy.Ipa.Wrap.challenges_computed` here:
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/wrap_hack.ml#L37
    ///
    /// Those are constants but they are computed once at runtime in Mina.
    /// TODO: Compute them instead of hardcoded values
    fn dummy_padding() -> [Fq; 15] {
        let f = |s| Fq::from_str(s).unwrap();

        [
            f("7048930911355605315581096707847688535149125545610393399193999502037687877674"),
            f("5945064094191074331354717685811267396540107129706976521474145740173204364019"),
            f("20315491820009986698838977727629973056499886675589920515484193128018854963801"),
            f("375929229548289966749422550601268097380795636681684498450629863247980915833"),
            f("19682218496321100578766622300447982536359891434050417209656101638029891689955"),
            f("516598185966802396400068849903674663130928531697254466925429658676832606723"),
            f("23729760760563685146228624125180554011222918208600079938584869191222807389336"),
            f("11155777282048225577422475738306432747575091690354122761439079853293714987855"),
            f("24977767586983413450834833875715786066408803952857478894197349635213480783870"),
            f("2813347787496113574506936084777563965225649411532015639663405402448028142689"),
            f("22626141769059119580550800305467929090916842064220293932303261732461616709448"),
            f("18748107085456859495495117012311103043200881556220793307463332157672741458218"),
            f("22196219950929618042921320796106738233125483954115679355597636800196070731081"),
            f("13054421325261400802177761929986025883530654947859503505174678618288142017333"),
            f("4799483385651443229337780097631636300491234601736019220096005875687579936102"),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct MessagesForNextStepProof<'a, AppState: ToFieldElements<Fp>> {
    pub app_state: &'a AppState,
    pub dlog_plonk_index: &'a PlonkVerificationKeyEvals,
    pub challenge_polynomial_commitments: Vec<CurveAffine<Fp>>,
    pub old_bulletproof_challenges: Vec<[Fp; 16]>,
}

impl<AppState> MessagesForNextStepProof<'_, AppState>
where
    AppState: ToFieldElements<Fp>,
{
    /// Implementation of `hash_messages_for_next_step_proof`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/common.ml#L33
    pub fn hash(&self) -> [u64; 4] {
        let fields: Vec<Fp> = self.to_fields();
        let field: Fp = hash_fields(&fields);

        let bigint: BigInteger256 = field.into_repr();
        bigint.0
    }

    /// Implementation of `to_field_elements`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/composition_types/composition_types.ml#L493
    fn to_fields(&self) -> Vec<Fp> {
        const NFIELDS: usize = 93; // TODO: This is bigger with transactions

        let mut fields = Vec::with_capacity(NFIELDS);

        // Self::dlog_plonk_index
        // Refactor with `src/account/account.rs`, this is the same code
        {
            let index = &self.dlog_plonk_index;

            for field in index.sigma {
                fields.push(field.0);
                fields.push(field.1);
            }

            for field in index.coefficients {
                fields.push(field.0);
                fields.push(field.1);
            }

            fields.push(index.generic.0);
            fields.push(index.generic.1);

            fields.push(index.psm.0);
            fields.push(index.psm.1);

            fields.push(index.complete_add.0);
            fields.push(index.complete_add.1);

            fields.push(index.mul.0);
            fields.push(index.mul.1);

            fields.push(index.emul.0);
            fields.push(index.emul.1);

            fields.push(index.endomul_scalar.0);
            fields.push(index.endomul_scalar.1);
        }

        self.app_state.to_field_elements(&mut fields);

        // Self::challenge_polynomial_commitments and Self::old_bulletproof_challenges
        let commitments = &self.challenge_polynomial_commitments;
        let old_challenges = &self.old_bulletproof_challenges;
        for (commitments, old) in commitments.iter().zip(old_challenges) {
            fields.push(commitments.0);
            fields.push(commitments.1);
            fields.extend_from_slice(old);
        }

        // assert!(fields.len() >= NFIELDS);

        fields
    }
}
