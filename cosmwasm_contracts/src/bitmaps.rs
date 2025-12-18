use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint256;

#[cw_serde]
pub struct BitMap256 {
    pub data: Uint256,
}

impl BitMap256 {
    pub fn zero() -> Self {
        Self {
            data: Uint256::zero(),
        }
    }

    pub fn from_u128(value: u128) -> Self {
        Self {
            data: Uint256::from(value),
        }
    }

    fn pow2(index: u32) -> Uint256 {
        let mut v = Uint256::from(1u128);
        for _ in 0..index {
            v = v.checked_mul(Uint256::from(2u128)).unwrap();
        }
        v
    }

    pub fn get(&self, index: u32) -> bool {
        assert!(index < 256, "bitmap index out of range");

        let mask = Self::pow2(index);

        // bit is set if:  data / mask  is odd
        !self.data.is_zero()
            && (self.data.checked_div(mask).unwrap() % Uint256::from(2u128) == Uint256::one())
    }

    pub fn set(&mut self, index: u32) {
        assert!(index < 256, "bitmap index out of range");

        let mask = Self::pow2(index);

        if !self.get(index) {
            self.data = self.data.checked_add(mask).unwrap();
        }
    }

    pub fn unset(&mut self, index: u32) {
        assert!(index < 256, "bitmap index out of range");

        let mask = Self::pow2(index);

        if self.get(index) {
            self.data = self.data.checked_sub(mask).unwrap();
        }
    }

    pub fn set_to(&mut self, index: u32, value: bool) {
        if value {
            self.set(index);
        } else {
            self.unset(index);
        }
    }

    pub fn member_count_up_to(&self, up_to: u32) -> u32 {
        assert!(up_to <= 256);
        let mut count = 0;
        for idx in 0..up_to {
            if self.get(idx) {
                count += 1;
            }
        }
        count
    }

    pub fn is_zero(&self) -> bool {
        self.data.is_zero()
    }
}

impl Default for BitMap256 {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<Uint256> for BitMap256 {
    fn from(value: Uint256) -> Self {
        Self { data: value }
    }
}

impl From<BitMap256> for Uint256 {
    fn from(value: BitMap256) -> Self {
        value.data
    }
}
