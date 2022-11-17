use std::{borrow::Cow, io::Cursor, str::FromStr};

use ark_ff::{Field, One, UniformRand, Zero};
use binprot::{BinProtRead, BinProtWrite};
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;
use rand::{prelude::ThreadRng, Rng};

use crate::hash::{hash_noinputs, hash_with_kimchi, Inputs};

use super::common::*;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TokenId(pub Fp);

impl std::fmt::Debug for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TokenId").field(&self.0.to_string()).finish()
    }
}

impl Default for TokenId {
    fn default() -> Self {
        Self(Fp::one())
    }
}

impl From<u64> for TokenId {
    fn from(num: u64) -> Self {
        TokenId(Fp::from(num))
    }
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account.ml#L93
pub type TokenSymbol = String;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L49
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permissions<Controller> {
    pub edit_state: Controller,
    pub send: Controller,
    pub receive: Controller,
    pub set_delegate: Controller,
    pub set_permissions: Controller,
    pub set_verification_key: Controller,
    pub set_zkapp_uri: Controller,
    pub edit_sequence_state: Controller,
    pub set_token_symbol: Controller,
    pub increment_nonce: Controller,
    pub set_voting_for: Controller,
}

impl Default for Permissions<AuthRequired> {
    fn default() -> Self {
        Self::user_default()
    }
}

impl Permissions<AuthRequired> {
    pub fn user_default() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: Signature,
            send: Signature,
            receive: None,
            set_delegate: Signature,
            set_permissions: Signature,
            set_verification_key: Signature,
            set_zkapp_uri: Signature,
            edit_sequence_state: Signature,
            set_token_symbol: Signature,
            increment_nonce: Signature,
            set_voting_for: Signature,
        }
    }

    pub fn empty() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: None,
            send: None,
            receive: None,
            set_delegate: None,
            set_permissions: None,
            set_verification_key: None,
            set_zkapp_uri: None,
            edit_sequence_state: None,
            set_token_symbol: None,
            increment_nonce: None,
            set_voting_for: None,
        }
    }
}

// TODO: Not sure if the name is correct
// It seems that a similar type exist in proof-systems: TODO
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CurveAffine<F: Field>(pub F, pub F);

impl<F> CurveAffine<F>
where
    F: Field + UniformRand + From<i32>,
{
    pub fn rand(rng: &mut ThreadRng) -> Self {
        let a = F::rand(rng);
        let two: F = 2.into();
        let b: F = a.mul(two);

        Self(a, b)
    }
}

// https://github.com/MinaProtocol/mina/blob/a6e5f182855b3f4b4afb0ea8636760e618e2f7a0/src/lib/pickles_types/plonk_verification_key_evals.ml#L9-L18
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlonkVerificationKeyEvals {
    pub sigma: [CurveAffine<Fp>; 7],
    pub coefficients: [CurveAffine<Fp>; 15],
    pub generic: CurveAffine<Fp>,
    pub psm: CurveAffine<Fp>,
    pub complete_add: CurveAffine<Fp>,
    pub mul: CurveAffine<Fp>,
    pub emul: CurveAffine<Fp>,
    pub endomul_scalar: CurveAffine<Fp>,
} // 28 CurveAffine, 56 Fp

impl PlonkVerificationKeyEvals {
    pub fn rand(rng: &mut ThreadRng) -> Self {
        Self {
            sigma: [
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
            ],
            coefficients: [
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
                CurveAffine::rand(rng),
            ],
            generic: CurveAffine::rand(rng),
            psm: CurveAffine::rand(rng),
            complete_add: CurveAffine::rand(rng),
            mul: CurveAffine::rand(rng),
            emul: CurveAffine::rand(rng),
            endomul_scalar: CurveAffine::rand(rng),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofVerified {
    N0,
    N1,
    N2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationKey {
    pub max_proofs_verified: ProofVerified,
    pub wrap_index: PlonkVerificationKeyEvals,
    // `wrap_vk` is not used for hash inputs
    pub wrap_vk: Option<()>,
}

impl VerificationKey {
    // https://github.com/MinaProtocol/mina/blob/35b1702fbc295713f9bb46bb17e2d007bc2bab84/src/lib/pickles/side_loaded_verification_key.ml#L295-L309
    fn dummy() -> Self {
        let g = CurveAffine(
            Fp::one(),
            Fp::from_str(
                "12418654782883325593414442427049395787963493412651469444558597405572177144507",
            )
            .unwrap(),
        );
        Self {
            max_proofs_verified: ProofVerified::N2,
            wrap_index: PlonkVerificationKeyEvals {
                sigma: [g; 7],
                coefficients: [g; 15],
                generic: g,
                psm: g,
                complete_add: g,
                mul: g,
                emul: g,
                endomul_scalar: g,
            },
            wrap_vk: None,
        }
    }

    fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        // https://github.com/MinaProtocol/mina/blob/35b1702fbc295713f9bb46bb17e2d007bc2bab84/src/lib/pickles_base/proofs_verified.ml#L108-L118
        let bits = match self.max_proofs_verified {
            ProofVerified::N0 => [true, false, false],
            ProofVerified::N1 => [false, true, false],
            ProofVerified::N2 => [false, false, true],
        };

        for bit in bits {
            inputs.append_bool(bit);
        }

        let index = &self.wrap_index;

        for field in index.sigma {
            inputs.append_field(field.0);
            inputs.append_field(field.1);
        }

        for field in index.coefficients {
            inputs.append_field(field.0);
            inputs.append_field(field.1);
        }

        inputs.append_field(index.generic.0);
        inputs.append_field(index.generic.1);

        inputs.append_field(index.psm.0);
        inputs.append_field(index.psm.1);

        inputs.append_field(index.complete_add.0);
        inputs.append_field(index.complete_add.1);

        inputs.append_field(index.mul.0);
        inputs.append_field(index.mul.1);

        inputs.append_field(index.emul.0);
        inputs.append_field(index.emul.1);

        inputs.append_field(index.endomul_scalar.0);
        inputs.append_field(index.endomul_scalar.1);

        hash_with_kimchi("MinaSideLoadedVk", &inputs.to_fields())
    }
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/zkapp_account.ml#L148-L170
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZkAppAccount {
    pub app_state: [Fp; 8],
    pub verification_key: Option<VerificationKey>,
    pub zkapp_version: u32,
    pub sequence_state: [Fp; 5],
    pub last_sequence_slot: Slot,
    pub proved_state: bool,
    pub zkapp_uri: String,
}

impl Default for ZkAppAccount {
    fn default() -> Self {
        Self {
            app_state: [Fp::zero(); 8],
            verification_key: None,
            zkapp_version: 0,
            sequence_state: {
                let empty = hash_noinputs("MinaZkappSequenceStateEmptyElt");
                [empty, empty, empty, empty, empty]
            },
            last_sequence_slot: 0,
            proved_state: false,
            zkapp_uri: String::new(),
        }
    }
}

#[derive(Clone, Eq)]
pub struct AccountId {
    pub public_key: CompressedPubKey,
    pub token_id: TokenId,
}

impl std::fmt::Debug for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pubkey = self.public_key.x.to_string();
        f.debug_struct("AccountId")
            .field("public_key", &pubkey)
            .field("token_id", &self.token_id)
            .finish()
    }
}

impl std::hash::Hash for AccountId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.public_key.x.hash(state);
        self.public_key.is_odd.hash(state);
        self.token_id.hash(state);
    }
}

impl PartialEq for AccountId {
    fn eq(&self, other: &Self) -> bool {
        self.public_key.x == other.public_key.x
            && self.public_key.is_odd == other.public_key.is_odd
            && self.token_id.0 == other.token_id.0
    }
}

// https://github.com/MinaProtocol/mina/blob/1765ba6bdfd7c454e5ae836c49979fa076de1bea/src/lib/mina_base/account.ml#L368
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub public_key: CompressedPubKey,         // Public_key.Compressed.t
    pub token_id: TokenId,                    // Token_id.t
    pub token_permissions: TokenPermissions,  // Token_permissions.t
    pub token_symbol: TokenSymbol,            // Token_symbol.t
    pub balance: Balance,                     // Balance.t
    pub nonce: Nonce,                         // Nonce.t
    pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
    pub delegate: Option<CompressedPubKey>,   // Public_key.Compressed.t option
    pub voting_for: VotingFor,                // State_hash.t
    pub timing: Timing,                       // Timing.t
    pub permissions: Permissions<AuthRequired>, // Permissions.t
    pub zkapp: Option<ZkAppAccount>,          // Zkapp_account.t
}

impl Account {
    pub fn create() -> Self {
        let pubkey = CompressedPubKey::from_address(
            "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
        )
        .unwrap();

        Self {
            public_key: pubkey.clone(),
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::default(),
            token_symbol: String::new(),
            balance: 10101,
            nonce: 0,
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: Some(pubkey),
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
        }
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);
        Account::binprot_read(&mut cursor).unwrap()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(10000);
        self.binprot_write(&mut bytes).unwrap();
        bytes
    }

    pub fn empty() -> Self {
        Self {
            public_key: CompressedPubKey {
                x: Fp::zero(),
                is_odd: false,
            },
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::default(),
            token_symbol: String::new(),
            balance: 0,
            nonce: 0,
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: None,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
        }
    }

    pub fn id(&self) -> AccountId {
        AccountId {
            public_key: self.public_key.clone(),
            token_id: self.token_id.clone(),
        }
    }

    pub fn hash(&self) -> Fp {
        // println!("account={:#?}", self);

        let mut inputs = Inputs::new();

        // Self::zkapp
        let field_zkapp = {
            let zkapp = match self.zkapp.as_ref() {
                Some(zkapp) => Cow::Borrowed(zkapp),
                None => Cow::Owned(ZkAppAccount::default()),
            };
            let zkapp = zkapp.as_ref();

            let mut inputs = Inputs::new();

            // Self::zkapp_uri
            // Note: This doesn't cover when zkapp_uri is None, which
            // is never the case for accounts
            let field_zkapp_uri = {
                let mut inputs = Inputs::new();

                for c in zkapp.zkapp_uri.as_bytes() {
                    for j in 0..8 {
                        inputs.append_bool((c & (1 << j)) != 0);
                    }
                }
                inputs.append_bool(true);

                hash_with_kimchi("MinaZkappUri", &inputs.to_fields())
            };

            inputs.append_field(field_zkapp_uri);

            inputs.append_bool(zkapp.proved_state);
            inputs.append_u32(zkapp.last_sequence_slot);
            for fp in &zkapp.sequence_state {
                inputs.append_field(*fp);
            }
            inputs.append_u32(zkapp.zkapp_version);
            let vk_hash = match zkapp.verification_key.as_ref() {
                Some(vk) => vk.hash(),
                None => VerificationKey::dummy().hash(),
            };
            inputs.append_field(vk_hash);
            for fp in &zkapp.app_state {
                inputs.append_field(*fp);
            }

            hash_with_kimchi("MinaZkappAccount", &inputs.to_fields())
        };

        inputs.append_field(field_zkapp);

        // Self::permissions
        for auth in [
            self.permissions.edit_state,
            self.permissions.send,
            self.permissions.receive,
            self.permissions.set_delegate,
            self.permissions.set_permissions,
            self.permissions.set_verification_key,
            self.permissions.set_zkapp_uri,
            self.permissions.edit_sequence_state,
            self.permissions.set_token_symbol,
            self.permissions.increment_nonce,
            self.permissions.set_voting_for,
        ] {
            for bit in auth.encode().to_bits() {
                inputs.append_bool(bit);
            }
        }

        // Self::timing
        match self.timing {
            Timing::Untimed => {
                inputs.append_bool(false);
                inputs.append_u64(0); // initial_minimum_balance
                inputs.append_u32(0); // cliff_time
                inputs.append_u64(0); // cliff_amount
                inputs.append_u32(1); // vesting_period
                inputs.append_u64(0); // vesting_increment
            }
            Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => {
                inputs.append_bool(true);
                inputs.append_u64(initial_minimum_balance);
                inputs.append_u32(cliff_time);
                inputs.append_u64(cliff_amount);
                inputs.append_u32(vesting_period);
                inputs.append_u64(vesting_increment);
            }
        }

        // Self::voting_for
        inputs.append_field(self.voting_for.0);

        // Self::delegate
        match self.delegate.as_ref() {
            Some(delegate) => {
                inputs.append_field(delegate.x);
                inputs.append_bool(delegate.is_odd);
            }
            None => {
                // Public_key.Compressed.empty
                inputs.append_field(Fp::zero());
                inputs.append_bool(false);
            }
        }

        // Self::receipt_chain_hash
        inputs.append_field(self.receipt_chain_hash.0);

        // Self::nonce
        inputs.append_u32(self.nonce);

        // Self::balance
        inputs.append_u64(self.balance);

        // Self::token_symbol

        // https://github.com/MinaProtocol/mina/blob/2fac5d806a06af215dbab02f7b154b4f032538b7/src/lib/mina_base/account.ml#L97
        assert!(self.token_symbol.len() <= 6);

        let mut s = <[u8; 6]>::default();
        if !self.token_symbol.is_empty() {
            let len = self.token_symbol.len();
            s[..len].copy_from_slice(self.token_symbol.as_bytes());
        }
        inputs.append_u48(s);

        // Self::token_permissions
        match self.token_permissions {
            TokenPermissions::TokenOwned {
                disable_new_accounts,
            } => {
                let bits = if disable_new_accounts { 0b10 } else { 0b00 };
                inputs.append_u2(0b01 | bits);
            }
            TokenPermissions::NotOwned { account_disabled } => {
                let bits = if account_disabled { 0b10 } else { 0b00 };
                inputs.append_u2(bits);
            }
        }

        // Self::token_id
        inputs.append_field(self.token_id.0);

        // Self::public_key
        inputs.append_field(self.public_key.x);
        inputs.append_bool(self.public_key.is_odd);

        hash_with_kimchi("MinaAccount", &inputs.to_fields())
    }

    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();
        let rng = &mut rng;

        let symbol: u64 = rng.gen();
        let mut symbol = symbol.to_string();
        symbol.truncate(6);

        let zkapp_uri: u64 = rng.gen();
        let mut zkapp_uri = zkapp_uri.to_string();
        zkapp_uri.truncate(6);

        let gen_perm = |rng: &mut ThreadRng| {
            let n: u64 = rng.gen();
            if n % 5 == 0 {
                AuthRequired::Either
            } else if n % 4 == 0 {
                AuthRequired::Impossible
            } else if n % 3 == 0 {
                AuthRequired::None
            } else if n % 2 == 0 {
                AuthRequired::Proof
            } else {
                AuthRequired::Signature
            }
        };

        Self {
            public_key: CompressedPubKey {
                x: Fp::rand(rng),
                is_odd: rng.gen(),
            },
            token_id: TokenId(Fp::rand(rng)),
            token_permissions: if rng.gen() {
                TokenPermissions::NotOwned {
                    account_disabled: rng.gen(),
                }
            } else {
                TokenPermissions::TokenOwned {
                    disable_new_accounts: rng.gen(),
                }
            },
            token_symbol: symbol,
            balance: rng.gen(),
            nonce: rng.gen(),
            receipt_chain_hash: ReceiptChainHash(Fp::rand(rng)),
            delegate: if rng.gen() {
                Some(CompressedPubKey {
                    x: Fp::rand(rng),
                    is_odd: rng.gen(),
                })
            } else {
                None
            },
            voting_for: VotingFor(Fp::rand(rng)),
            timing: if rng.gen() {
                Timing::Untimed
            } else {
                Timing::Timed {
                    initial_minimum_balance: rng.gen(),
                    cliff_time: rng.gen(),
                    cliff_amount: rng.gen(),
                    vesting_period: rng.gen(),
                    vesting_increment: rng.gen(),
                }
            },
            permissions: Permissions {
                edit_state: gen_perm(rng),
                send: gen_perm(rng),
                receive: gen_perm(rng),
                set_delegate: gen_perm(rng),
                set_permissions: gen_perm(rng),
                set_verification_key: gen_perm(rng),
                set_zkapp_uri: gen_perm(rng),
                edit_sequence_state: gen_perm(rng),
                set_token_symbol: gen_perm(rng),
                increment_nonce: gen_perm(rng),
                set_voting_for: gen_perm(rng),
            },
            zkapp: if rng.gen() {
                Some(ZkAppAccount {
                    app_state: [
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                    ],
                    verification_key: if rng.gen() {
                        Some(VerificationKey {
                            max_proofs_verified: {
                                let n: u64 = rng.gen();

                                if n % 3 == 0 {
                                    ProofVerified::N2
                                } else if n % 2 == 0 {
                                    ProofVerified::N1
                                } else {
                                    ProofVerified::N0
                                }
                            },
                            wrap_index: PlonkVerificationKeyEvals::rand(rng),
                            wrap_vk: None,
                        })
                    } else {
                        None
                    },
                    zkapp_version: rng.gen(),
                    sequence_state: [
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                    ],
                    last_sequence_slot: rng.gen(),
                    proved_state: rng.gen(),
                    zkapp_uri,
                })
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use o1_utils::FieldHelpers;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[cfg(not(target_family = "wasm"))]
    use crate::{base::BaseLedger, database::Database, tree_version::V2};

    use super::*;

    #[test]
    fn test_size_account() {
        #[cfg(not(target_family = "wasm"))]
        const SIZE: usize = 2528;

        #[cfg(target_family = "wasm")]
        const SIZE: usize = 2504;

        assert_eq!(std::mem::size_of::<Account>(), SIZE);
    }

    #[test]
    fn test_hash_account() {
        let acc = Account::create();
        let hash = acc.hash();

        println!("account_hash={}", hash);
        println!("account_hash={}", hash.to_hex());

        assert_eq!(
            hash.to_hex(),
            "82a455aa81f57fca2f0b40662ecd6ee6a73dc181fda2f0e233d35813cc5b2b12"
        );

        let acc = Account {
            public_key: CompressedPubKey::from_address(
                "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
            )
            .unwrap(),
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::default(),
            token_symbol: "seb".to_string(),
            balance: 10101,
            nonce: 62772,
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: None,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
        };

        assert_eq!(
            acc.hash().to_hex(),
            "fc040a2d79358b092265687701b182b5e32eb000b47d0fa7e394cb8193086d2e"
        );
    }

    #[test]
    fn test_dummy_sideloaded_verification_key() {
        assert_eq!(
            VerificationKey::dummy().hash().to_hex(),
            "b5d8852f07bb6daffbc4a68829141643f56ebd86a2a571e9d0e939e929fba40f"
        );
    }

    #[test]
    fn test_from_deserialized_account() {
        let bytes: &[u8] = &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 0, 3, 115, 101, 98, 0, 0, 155, 228, 183, 197, 30, 217, 194,
            228, 82, 71, 39, 128, 95, 211, 111, 82, 32, 251, 252, 112, 167, 73, 246, 38, 35, 176,
            237, 41, 8, 67, 51, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0,
        ];

        // This deserialize to `MinaBaseAccountBinableArgStableV2` and convert to `Account`
        let acc: Account = Account::deserialize(bytes);

        assert_eq!(
            acc.hash().to_hex(),
            "2e03fd55707e58f82cf5d57ace864ca8cb50a10744d0b127905c0e623bf27214"
        );

        let bytes = &[
            176, 194, 45, 223, 254, 30, 162, 197, 122, 221, 132, 151, 117, 60, 70, 134, 41, 158,
            116, 38, 124, 102, 236, 184, 238, 131, 107, 151, 247, 248, 28, 18, 0, 149, 229, 111,
            200, 171, 208, 82, 180, 2, 73, 133, 192, 69, 102, 234, 26, 240, 98, 220, 178, 144, 145,
            39, 106, 68, 31, 62, 115, 153, 45, 252, 11, 1, 0, 0, 252, 27, 35, 154, 15, 127, 164,
            201, 170, 0, 155, 228, 183, 197, 30, 217, 194, 228, 82, 71, 39, 128, 95, 211, 111, 82,
            32, 251, 252, 112, 167, 73, 246, 38, 35, 176, 237, 41, 8, 67, 51, 32, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
            3, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0,
        ];
        let acc: Account = Account::deserialize(bytes);

        assert_eq!(
            acc.hash().to_hex(),
            "1f084f56133cb3735f3e6ffc3dda945a6e8446fef2746662bae99d8249ebcc16"
        );

        let fp = Fp::from_str(
            "6989982961557644252722402794378511163775946371102905721368942795880969184859",
        )
        .unwrap();
        println!("FP={:?}", fp.to_string());

        let bytes = &[
            178, 29, 73, 50, 85, 80, 131, 166, 53, 11, 48, 224, 103, 89, 161, 207, 149, 31, 170,
            21, 165, 181, 94, 18, 149, 177, 54, 71, 185, 77, 109, 49, 1, 144, 247, 164, 171, 110,
            24, 3, 12, 25, 163, 63, 125, 83, 66, 174, 2, 160, 62, 45, 137, 185, 47, 16, 129, 145,
            190, 203, 124, 35, 119, 251, 26, 1, 1, 6, 49, 50, 56, 54, 56, 56, 252, 29, 154, 218,
            214, 79, 98, 177, 181, 253, 181, 152, 127, 0, 145, 177, 91, 155, 59, 239, 161, 174,
            217, 42, 201, 30, 46, 11, 187, 88, 49, 5, 111, 254, 222, 87, 42, 45, 90, 1, 236, 173,
            205, 215, 241, 20, 0, 77, 12, 197, 234, 69, 202, 22, 55, 50, 183, 255, 238, 8, 29, 79,
            199, 92, 12, 146, 223, 105, 45, 135, 77, 89, 73, 141, 11, 137, 28, 54, 21, 0, 1, 4, 4,
            1, 0, 4, 3, 4, 3, 2, 3, 0, 6, 49, 49, 56, 54, 54, 51,
        ];
        let acc: Account = Account::deserialize(bytes);

        println!("ACC={:#?}", acc);

        let h = acc.hash();
        println!("HASH={:?}", h.to_string());

        assert_eq!(
            acc.hash().to_hex(),
            "7e820d3d22f7406151f0f031ab509cc326eba01f447f3e21a74f41fbcdf89714"
        );

        // let fp = Fp::from_str(
        //     "6989982961557644252722402794378511163775946371102905721368942795880969184859",
        // )
        // .unwrap();
        // println!("FP={:?}", fp.to_string());
    }

    #[test]
    fn test_rand() {
        for _ in 0..1000 {
            let rand = Account::rand();
            let hash = rand.hash();

            let bytes = Account::serialize(&rand);
            let rand2: Account = Account::deserialize(&bytes);

            assert_eq!(hash, rand2.hash());
        }
    }

    #[cfg(not(target_family = "wasm"))] // Use multiple threads
    #[test]
    fn test_rand_tree() {
        use rayon::prelude::*;

        let mut db = Database::<V2>::create(20);
        let mut accounts = Vec::with_capacity(1000);

        const NACCOUNTS: usize = 1000;

        for _ in 0..NACCOUNTS {
            let rand = Account::rand();
            accounts.push(rand);
        }

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(16)
            .build()
            .unwrap();

        let now = std::time::Instant::now();
        let hashes = pool.install(|| {
            accounts
                .par_iter()
                .map(|acc| acc.hash())
                .collect::<Vec<_>>()
        });

        assert_eq!(hashes.len(), NACCOUNTS);
        println!(
            "elapsed to hash accounts in 16 threads: {:?}",
            now.elapsed(),
        );

        let mut hashes = Vec::with_capacity(accounts.len());
        let now = std::time::Instant::now();
        for account in accounts.iter() {
            hashes.push(account.hash());
        }
        assert_eq!(hashes.len(), NACCOUNTS);
        println!("elapsed to hash accounts in 1 thread: {:?}", now.elapsed(),);

        let now = std::time::Instant::now();
        for account in accounts.into_iter() {
            let id = account.id();
            db.get_or_create_account(id, account).unwrap();
        }
        assert_eq!(db.naccounts(), NACCOUNTS);
        println!("elapsed to insert in tree: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        db.root_hash();
        println!("root hash computed in {:?}", now.elapsed());
    }
}
