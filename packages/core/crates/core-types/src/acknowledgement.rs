use crate::acknowledgement::PendingAcknowledgement::{WaitingAsRelayer, WaitingAsSender};
use crate::channels::Ticket;
use core_crypto::errors::CryptoError::SignatureVerification;
use core_crypto::primitives::{DigestLike, SimpleDigest};
use core_crypto::types::{HalfKey, HalfKeyChallenge, Hash, PublicKey, Response, Signature};
use serde::{Deserialize, Serialize};
use utils_types::errors;
use utils_types::errors::GeneralError::ParseError;
use utils_types::traits::BinarySerializable;

/// Represents packet acknowledgement
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Acknowledgement {
    ack_signature: Signature,
    challenge_signature: Signature,
    pub ack_key_share: HalfKey,
    validated: bool,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Acknowledgement {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ack_challenge: AcknowledgementChallenge, ack_key_share: HalfKey, private_key: &[u8]) -> Self {
        let mut digest = SimpleDigest::default();
        digest.update(&ack_challenge.to_bytes());
        digest.update(&ack_key_share.to_bytes());

        Self {
            ack_signature: Signature::sign_hash(&digest.finalize(), private_key),
            challenge_signature: ack_challenge.signature,
            ack_key_share,
            validated: true,
        }
    }

    /// Validates the acknowledgement. Must be called immediately after deserialization or otherwise
    /// any operations with the deserialized acknowledgment will panic.
    pub fn validate(&mut self, own_public_key: &PublicKey, sender_public_key: &PublicKey) -> bool {
        let mut digest = SimpleDigest::default();
        digest.update(&self.ack_key_share.to_challenge().to_bytes());
        self.validated = self
            .challenge_signature
            .verify_hash_with_pubkey(&digest.finalize(), own_public_key);

        digest.update(&self.challenge_signature.to_bytes());
        digest.update(&self.ack_key_share.to_bytes());
        self.validated = self.validated
            && self
                .ack_signature
                .verify_hash_with_pubkey(&digest.finalize(), sender_public_key);

        self.validated
    }

    /// Obtains the acknowledged challenge out of this acknowledgment.
    pub fn ack_challenge(&self) -> HalfKeyChallenge {
        assert!(self.validated, "acknowledgement not validated");
        self.ack_key_share.to_challenge()
    }
}

impl BinarySerializable<'_> for Acknowledgement {
    const SIZE: usize = Signature::SIZE + AcknowledgementChallenge::SIZE + HalfKey::SIZE;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        let mut buf = data.to_vec();
        if data.len() == Self::SIZE {
            let ack_signature = Signature::from_bytes(buf.drain(..Signature::SIZE).as_ref())?;
            let challenge_signature =
                AcknowledgementChallenge::from_bytes(buf.drain(..AcknowledgementChallenge::SIZE).as_ref())?;
            let ack_key_share = HalfKey::from_bytes(buf.drain(..HalfKey::SIZE).as_ref())?;
            Ok(Self {
                ack_signature,
                challenge_signature: challenge_signature.signature,
                ack_key_share,
                validated: false,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        assert!(self.validated, "acknowledgement not validated");
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ack_signature.raw_signature());
        ret.extend_from_slice(&self.challenge_signature.raw_signature());
        ret.extend_from_slice(&self.ack_key_share.to_bytes());
        ret.into_boxed_slice()
    }
}

/// Contains acknowledgment information and the respective ticket
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AcknowledgedTicket {
    pub ticket: Ticket,
    pub response: Response,
    pub pre_image: Hash,
    pub signer: PublicKey,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AcknowledgedTicket {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ticket: Ticket, response: Response, pre_image: Hash, signer: PublicKey) -> Self {
        assert_ne!(
            ticket.counterparty,
            signer.to_address(),
            "signer must be different from the ticket counterparty"
        );
        Self {
            ticket,
            response,
            pre_image,
            signer,
        }
    }

    pub fn set_preimage(&mut self, hash: &Hash) {
        self.pre_image = hash.clone();
    }
}

impl BinarySerializable<'_> for AcknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + Response::SIZE + Hash::SIZE + PublicKey::SIZE_COMPRESSED;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let ticket = Ticket::from_bytes(buf.drain(..Ticket::SIZE).as_ref())?;
            let response = Response::from_bytes(buf.drain(..Response::SIZE).as_ref())?;
            let pre_image = Hash::from_bytes(buf.drain(..Hash::SIZE).as_ref())?;
            let signer = PublicKey::from_bytes(buf.drain(..PublicKey::SIZE_COMPRESSED).as_ref())?;

            Ok(Self {
                ticket,
                response,
                pre_image,
                signer,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.extend_from_slice(&self.response.to_bytes());
        ret.extend_from_slice(&self.pre_image.to_bytes());
        ret.extend_from_slice(&self.signer.to_bytes(true));
        ret.into_boxed_slice()
    }
}

impl AcknowledgedTicket {
    /// Verifies if the embedded ticket has been signed by the given issuer and also
    /// that the challenge on the embedded response matches the challenge on the ticket.
    pub fn verify(&self, issuer: &PublicKey) -> core_crypto::errors::Result<()> {
        (self.ticket.verify(issuer).map(|_| true)?
            && self
                .response
                .to_challenge()
                .to_ethereum_challenge()
                .eq(&self.ticket.challenge))
        .then_some(())
        .ok_or(SignatureVerification)
    }
}

/// Wrapper for an unacknowledged ticket
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct UnacknowledgedTicket {
    pub ticket: Ticket,
    pub own_key: HalfKey,
    pub signer: PublicKey,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl UnacknowledgedTicket {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ticket: Ticket, own_key: HalfKey, signer: PublicKey) -> Self {
        Self {
            ticket,
            own_key,
            signer,
        }
    }

    pub fn get_challenge(&self) -> HalfKeyChallenge {
        self.own_key.to_challenge()
    }
}

impl UnacknowledgedTicket {
    /// Verifies if signature on the embedded ticket using the embedded public key.
    pub fn verify_signature(&self) -> core_crypto::errors::Result<()> {
        self.ticket.verify(&self.signer)
    }

    /// Verifies if the challenge on the embedded ticket matches the solution
    /// from the given acknowledgement and the embedded half key.
    pub fn verify_challenge(&self, acknowledgement: &HalfKey) -> core_crypto::errors::Result<()> {
        self.get_response(acknowledgement)?
            .to_challenge()
            .to_ethereum_challenge()
            .eq(&self.ticket.challenge)
            .then(|| ())
            .ok_or(SignatureVerification)
    }

    pub fn get_response(&self, acknowledgement: &HalfKey) -> core_crypto::errors::Result<Response> {
        Response::from_half_keys(&self.own_key, acknowledgement)
    }
}

impl BinarySerializable<'_> for UnacknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + HalfKey::SIZE + PublicKey::SIZE_UNCOMPRESSED;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let ticket = Ticket::from_bytes(buf.drain(..Ticket::SIZE).as_ref())?;
            let own_key = HalfKey::from_bytes(buf.drain(..HalfKey::SIZE).as_ref())?;
            let signer = PublicKey::from_bytes(buf.drain(..PublicKey::SIZE_UNCOMPRESSED).as_ref())?;
            Ok(Self {
                ticket,
                own_key,
                signer,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.extend_from_slice(&self.own_key.to_bytes());
        ret.extend_from_slice(&self.signer.to_bytes(false));
        ret.into_boxed_slice()
    }
}

/// Contains cryptographic challenge that needs to be solved for acknowledging a packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct AcknowledgementChallenge {
    ack_challenge: Option<HalfKeyChallenge>,
    signature: Signature,
}

fn hash_challenge(challenge: &HalfKeyChallenge) -> Box<[u8]> {
    let mut digest = SimpleDigest::default();
    digest.update(&challenge.to_bytes());
    digest.finalize()
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AcknowledgementChallenge {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ack_challenge: &HalfKeyChallenge, private_key: &[u8]) -> Self {
        let hash = hash_challenge(&ack_challenge);
        Self {
            ack_challenge: Some(ack_challenge.clone()),
            signature: Signature::sign_hash(&hash, private_key),
        }
    }

    /// Checks if the given secret solves this challenge.
    pub fn solve(&self, secret: &[u8]) -> bool {
        self.ack_challenge
            .as_ref()
            .expect("challenge not valid")
            .eq(&HalfKey::new(secret).to_challenge())
    }

    pub fn verify(public_key: &PublicKey, signature: &Signature, challenge: &HalfKeyChallenge) -> bool {
        let hash = hash_challenge(challenge);
        signature.verify_hash_with_pubkey(&hash, public_key)
    }

    pub fn validate(&mut self, ack_challenge: HalfKeyChallenge, public_key: &PublicKey) -> bool {
        if self.ack_challenge.is_some() || Self::verify(public_key, &self.signature, &ack_challenge) {
            self.ack_challenge = Some(ack_challenge);
            true
        } else {
            false
        }
    }
}

impl BinarySerializable<'_> for AcknowledgementChallenge {
    const SIZE: usize = Signature::SIZE;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        if data.len() == Self::SIZE {
            Ok(AcknowledgementChallenge {
                ack_challenge: None,
                signature: Signature::from_bytes(data)?,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        assert!(self.ack_challenge.is_some(), "challenge is invalid");
        self.signature.raw_signature()
    }
}

/// Contains either unacknowledged ticket if we're waiting for the acknowledgement as a relayer
/// or information if we wait for the acknowledgement as a sender.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PendingAcknowledgement {
    /// We're waiting for acknowledgement as a sender
    WaitingAsSender,
    /// We're waiting for the acknowledgement as a relayer with a ticket
    WaitingAsRelayer(UnacknowledgedTicket),
}

impl PendingAcknowledgement {
    const SENDER_PREFIX: u8 = 0;
    const RELAYER_PREFIX: u8 = 1;
}

impl BinarySerializable<'_> for PendingAcknowledgement {
    const SIZE: usize = 1;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        if data.len() >= Self::SIZE {
            match data[0] {
                Self::SENDER_PREFIX => Ok(WaitingAsSender),
                Self::RELAYER_PREFIX => Ok(WaitingAsRelayer(UnacknowledgedTicket::from_bytes(&data[1..])?)),
                _ => Err(ParseError),
            }
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        match &self {
            WaitingAsSender => ret.push(Self::SENDER_PREFIX),
            WaitingAsRelayer(unacknowledged) => {
                ret.push(Self::RELAYER_PREFIX);
                ret.extend_from_slice(&unacknowledged.to_bytes());
            }
        }
        ret.into_boxed_slice()
    }
}

#[cfg(test)]
pub mod test {
    use crate::acknowledgement::{
        AcknowledgedTicket, Acknowledgement, AcknowledgementChallenge, PendingAcknowledgement, UnacknowledgedTicket,
    };
    use crate::channels::Ticket;
    use core_crypto::types::{Challenge, CurvePoint, HalfKey, Hash, PublicKey, Response};
    use ethnum::u256;
    use hex_literal::hex;
    use utils_types::primitives::{Address, Balance, BalanceType, U256};
    use utils_types::traits::BinarySerializable;

    fn mock_ticket(pk: &[u8]) -> Ticket {
        let inverse_win_prob = u256::new(1u128); // 100 %
        let price_per_packet = u256::new(10000000000000000u128); // 0.01 HOPR
        let path_pos = 5;

        Ticket::new(
            Address::new(&[0u8; Address::SIZE]),
            None,
            U256::new("1"),
            U256::new("2"),
            Balance::new(
                (inverse_win_prob * price_per_packet * path_pos as u128).into(),
                BalanceType::HOPR,
            ),
            U256::from_inverse_probability(&inverse_win_prob).unwrap(),
            U256::new("4"),
            pk,
        )
    }

    #[test]
    fn test_pending_ack_sender() {
        assert_eq!(
            PendingAcknowledgement::WaitingAsSender,
            PendingAcknowledgement::from_bytes(&PendingAcknowledgement::WaitingAsSender.to_bytes()).unwrap()
        );
    }

    #[test]
    fn test_acknowledgement_challenge() {
        let sk = hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa");
        let hkc = HalfKey::new(&sk).to_challenge();
        let pk = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
        let pub_key = PublicKey::from_privkey(&pk).unwrap();

        let mut akc1 = AcknowledgementChallenge::new(&hkc, &pk);
        assert!(akc1.validate(hkc.clone(), &pub_key));

        assert!(akc1.solve(&sk), "challenge must be solved by its own private key");

        AcknowledgementChallenge::verify(&PublicKey::from_privkey(&sk).unwrap(), &akc1.signature, &hkc);

        let mut akc2 = AcknowledgementChallenge::from_bytes(&akc1.to_bytes()).unwrap();
        assert!(akc2.validate(hkc.clone(), &pub_key));

        assert_eq!(akc1, akc2);
    }

    #[test]
    fn test_acknowledgement() {
        let pk_1 = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
        let pub_key_1 = PublicKey::from_privkey(&pk_1).unwrap();

        let pk_2 = hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b");
        let pub_key_2 = PublicKey::from_privkey(&pk_2).unwrap();

        let ack_key = HalfKey::new(&hex!(
            "3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"
        ));

        let akc1 = AcknowledgementChallenge::new(&ack_key.to_challenge(), &pk_1);

        let mut ack1 = Acknowledgement::new(akc1, ack_key, &pk_2);
        assert!(ack1.validate(&pub_key_1, &pub_key_2));

        let mut ack2 = Acknowledgement::from_bytes(&ack1.to_bytes()).unwrap();
        assert!(ack2.validate(&pub_key_1, &pub_key_2));

        assert_eq!(ack1, ack2);
    }

    #[test]
    fn test_unacknowledged_ticket() {
        let pk_1 = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
        let pub_key_1 = PublicKey::from_privkey(&pk_1).unwrap();

        let hk1 = HalfKey::new(&hex!(
            "3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"
        ));
        let hk2 = HalfKey::new(&hex!(
            "4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"
        ));
        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let mut ticket1 = mock_ticket(&pk_1);
        ticket1.set_challenge(Challenge::from(cp_sum).to_ethereum_challenge(), &pk_1);

        let unack1 = UnacknowledgedTicket::new(ticket1, hk1, pub_key_1);
        assert!(unack1.verify_signature().is_ok());
        assert!(unack1.verify_challenge(&hk2).is_ok());

        let unack2 = UnacknowledgedTicket::from_bytes(&unack1.to_bytes()).unwrap();
        assert_eq!(unack1, unack2);

        let pending_ack_1 = PendingAcknowledgement::WaitingAsRelayer(unack1);
        let pending_ack_2 = PendingAcknowledgement::from_bytes(&pending_ack_1.to_bytes()).unwrap();
        assert_eq!(pending_ack_1, pending_ack_2);
    }

    #[test]
    fn test_acknowledged_ticket() {
        let pk = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
        let pub_key = PublicKey::from_privkey(&pk).unwrap();
        let resp = Response::new(&hex!(
            "4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"
        ));

        let mut ticket1 = mock_ticket(&pk);
        ticket1.set_challenge(resp.to_challenge().to_ethereum_challenge(), &pk);

        let akt_1 = AcknowledgedTicket::new(ticket1, resp, Hash::create(&[&hex!("deadbeef")]), pub_key.clone());
        assert!(akt_1.verify(&pub_key).is_ok());

        let akt_2 = AcknowledgedTicket::from_bytes(&akt_1.to_bytes()).unwrap();
        assert_eq!(akt_1, akt_2);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::acknowledgement::{AcknowledgedTicket, Acknowledgement, AcknowledgementChallenge, UnacknowledgedTicket};
    use core_crypto::types::{HalfKey, PublicKey, Response};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::traits::BinarySerializable;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct PendingAcknowledgement {
        w: super::PendingAcknowledgement,
    }

    #[wasm_bindgen]
    impl PendingAcknowledgement {
        #[wasm_bindgen(constructor)]
        pub fn new(is_sender: bool, ticket: Option<UnacknowledgedTicket>) -> Self {
            if is_sender {
                Self {
                    w: super::PendingAcknowledgement::WaitingAsSender,
                }
            } else {
                Self {
                    w: super::PendingAcknowledgement::WaitingAsRelayer(ticket.unwrap()),
                }
            }
        }

        pub fn is_msg_sender(&self) -> bool {
            match &self.w {
                super::PendingAcknowledgement::WaitingAsSender => true,
                super::PendingAcknowledgement::WaitingAsRelayer(_) => false,
            }
        }

        pub fn ticket(&self) -> Option<UnacknowledgedTicket> {
            match &self.w {
                super::PendingAcknowledgement::WaitingAsSender => None,
                super::PendingAcknowledgement::WaitingAsRelayer(ticket) => Some(ticket.clone()),
            }
        }

        pub fn deserialize(data: &[u8]) -> JsResult<PendingAcknowledgement> {
            Ok(Self {
                w: ok_or_jserr!(super::PendingAcknowledgement::from_bytes(data))?,
            })
        }

        pub fn serialize(&self) -> Box<[u8]> {
            self.w.to_bytes()
        }
    }

    #[wasm_bindgen]
    impl UnacknowledgedTicket {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<UnacknowledgedTicket> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "get_response")]
        pub fn _get_response(&self, acknowledgement: &HalfKey) -> JsResult<Response> {
            ok_or_jserr!(self.get_response(acknowledgement))
        }

        #[wasm_bindgen(js_name = "verify_challenge")]
        pub fn _verify_challenge(&self, acknowledgement: &HalfKey) -> JsResult<bool> {
            ok_or_jserr!(self.verify_challenge(acknowledgement).map(|_| true))
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &UnacknowledgedTicket) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl AcknowledgedTicket {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<AcknowledgedTicket> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &AcknowledgedTicket) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "verify")]
        pub fn _verify(&self, issuer: &PublicKey) -> JsResult<bool> {
            ok_or_jserr!(self.verify(issuer).map(|_| true))
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl AcknowledgementChallenge {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<AcknowledgementChallenge> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &AcknowledgementChallenge) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Acknowledgement {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Acknowledgement> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Acknowledgement) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
