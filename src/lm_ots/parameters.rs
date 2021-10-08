use core::marker::PhantomData;

use arrayvec::ArrayVec;

use crate::{
    constants::MAX_HASH_SIZE,
    hasher::{sha256::Sha256Hasher, Hasher},
    util::coef::coef,
};

/// Specifies the used Winternitz parameter.
#[derive(Clone, Copy)]
pub enum LmotsAlgorithm {
    LmotsReserved = 0,
    LmotsW1 = 1,
    LmotsW2 = 2,
    LmotsW4 = 3,
    LmotsW8 = 4,
}

impl Default for LmotsAlgorithm {
    fn default() -> Self {
        LmotsAlgorithm::LmotsReserved
    }
}

impl From<u32> for LmotsAlgorithm {
    fn from(_type: u32) -> Self {
        match _type {
            1 => LmotsAlgorithm::LmotsW1,
            2 => LmotsAlgorithm::LmotsW2,
            3 => LmotsAlgorithm::LmotsW4,
            4 => LmotsAlgorithm::LmotsW8,
            _ => LmotsAlgorithm::LmotsReserved,
        }
    }
}

impl LmotsAlgorithm {
    pub fn construct_default_parameter() -> LmotsParameter {
        LmotsAlgorithm::LmotsW1.construct_parameter().unwrap()
    }

    pub fn construct_parameter<H: Hasher>(&self) -> Option<LmotsParameter<H>> {
        match *self {
            LmotsAlgorithm::LmotsReserved => None,
            LmotsAlgorithm::LmotsW1 => Some(LmotsParameter::new(1, 1, 265, 7)),
            LmotsAlgorithm::LmotsW2 => Some(LmotsParameter::new(2, 2, 133, 6)),
            LmotsAlgorithm::LmotsW4 => Some(LmotsParameter::new(3, 4, 67, 4)),
            LmotsAlgorithm::LmotsW8 => Some(LmotsParameter::new(4, 8, 34, 0)),
        }
    }

    pub fn get_from_type<H: Hasher>(_type: u32) -> Option<LmotsParameter<H>> {
        match _type {
            1 => LmotsAlgorithm::LmotsW1.construct_parameter(),
            2 => LmotsAlgorithm::LmotsW2.construct_parameter(),
            3 => LmotsAlgorithm::LmotsW4.construct_parameter(),
            4 => LmotsAlgorithm::LmotsW8.construct_parameter(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LmotsParameter<H: Hasher = Sha256Hasher> {
    type_id: u32,
    winternitz: u8,
    max_hash_iterations: u16,
    checksum_left_shift: u8,
    phantom_data: PhantomData<H>,
}

// Manually implement Copy because Hasher trait does not.
// However, it does not make a difference, because we don't hold a instance for Hasher.
impl<H: Hasher> Copy for LmotsParameter<H> {}

impl<H: Hasher> LmotsParameter<H> {
    const HASH_FUNCTION_OUTPUT_SIZE: usize = H::OUTPUT_SIZE as usize;

    pub fn new(
        type_id: u32,
        winternitz: u8,
        max_hash_iterations: u16,
        checksum_left_shift: u8,
    ) -> Self {
        Self {
            type_id,
            winternitz,
            max_hash_iterations,
            checksum_left_shift,
            phantom_data: PhantomData,
        }
    }

    pub fn get_type_id(&self) -> u32 {
        self.type_id
    }

    pub fn get_winternitz(&self) -> u8 {
        self.winternitz
    }

    pub fn get_max_hash_iterations(&self) -> u16 {
        self.max_hash_iterations
    }

    pub fn get_checksum_left_shift(&self) -> u8 {
        self.checksum_left_shift
    }

    pub fn get_hash_function_output_size(&self) -> usize {
        Self::HASH_FUNCTION_OUTPUT_SIZE
    }

    fn checksum(&self, byte_string: &[u8]) -> u16 {
        let mut sum = 0_u16;

        let max = (Self::HASH_FUNCTION_OUTPUT_SIZE as u16 * 8) / self.get_winternitz() as u16;

        let max_word_size: u64 = (1 << self.get_winternitz()) - 1;

        for i in 0..max {
            sum += (max_word_size - coef(byte_string, i, self.get_winternitz())) as u16;
        }

        sum << self.get_checksum_left_shift()
    }

    pub fn append_checksum_to(&self, byte_string: &[u8]) -> ArrayVec<u8, { MAX_HASH_SIZE + 2 }> {
        let mut result = ArrayVec::new();

        let checksum = self.checksum(byte_string);

        result.try_extend_from_slice(byte_string).unwrap();

        result
            .try_extend_from_slice(&[(checksum >> 8 & 0xff) as u8])
            .unwrap();
        result
            .try_extend_from_slice(&[(checksum & 0xff) as u8])
            .unwrap();

        result
    }

    pub fn get_hasher(&self) -> H {
        <H>::get_hasher()
    }
}

impl<H: Hasher> Default for LmotsParameter<H> {
    fn default() -> Self {
        LmotsAlgorithm::LmotsW1.construct_parameter().unwrap()
    }
}
