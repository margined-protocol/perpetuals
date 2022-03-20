use cosmwasm_std::{StdError, Uint128};
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::fmt::Write;
use std::str::FromStr;
use std::{cmp::Ordering, fmt};

/// Signed wrapper of Uint128
/// very minimalist only has bare minimum functions for
/// basic signed arithmetic
#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema)]
pub struct Integer {
    pub value: Uint128,
    pub negative: bool,
}

impl Integer {
    /// The maximum allowed
    pub const MAX: Integer = Integer {
        value: Uint128::MAX,
        negative: false,
    };

    /// The minimum allowed
    pub const MIN: Integer = Integer {
        value: Uint128::MAX,
        negative: true,
    };

    /// 0 as a Integer
    pub const ZERO: Integer = Integer {
        value: Uint128::zero(),
        negative: false,
    };

    /// create a new positive Integer with the given value
    pub fn new_positive<T: Into<Uint128>>(value: T) -> Self {
        Self {
            value: value.into(),
            negative: false,
        }
    }

    /// create a new negative Integer with the given value
    pub fn new_negative<T: Into<Uint128>>(value: T) -> Self {
        Self {
            value: value.into(),
            negative: true,
        }
    }

    /// turns positive to negative or negative to positive
    pub fn invert_sign(mut self) -> Self {
        self.negative = !self.negative;
        self
    }

    /// absolute value
    pub fn abs(mut self) -> Self {
        self.negative = false;
        self
    }

    #[allow(missing_docs)]
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    #[allow(missing_docs)]
    pub fn is_positive(&self) -> bool {
        !self.negative
    }

    #[allow(missing_docs)]
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

// Conversion
impl Default for Integer {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Uint128> for Integer {
    fn from(val: Uint128) -> Self {
        Self::new_positive(val)
    }
}

impl From<u128> for Integer {
    fn from(val: u128) -> Self {
        Self::new_positive(val)
    }
}

impl From<&str> for Integer {
    fn from(val: &str) -> Self {
        Integer::from_str(val).unwrap()
    }
}
impl From<String> for Integer {
    fn from(val: String) -> Self {
        Integer::from_str(&val).unwrap()
    }
}

impl FromStr for Integer {
    type Err = StdError;

    /// Converts the decimal string to an Integer, will default to positive
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        fn u128_from_str(input: &str) -> Result<u128, StdError> {
            match input.parse::<u128>() {
                Ok(u) => Ok(u),
                _ => Err(StdError::generic_err("Parsing u128".to_string())),
            }
        }

        match &input[..1] {
            "-" => {
                let value = u128_from_str(&input[1..])?;
                Ok(Integer::new_negative(value))
            }
            _ => {
                let value = u128_from_str(input)?;
                Ok(Integer::new_positive(value))
            }
        }
    }
}

// Display

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let whole = self.value;

        if self.negative && !whole.is_zero() {
            f.write_char('-')?;
        }

        f.write_str(&whole.to_string())?;
        Ok(())
    }
}

// Operations

impl std::ops::Mul for Integer {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let abs_value = self.value * rhs.value;

        match (self.negative, rhs.negative) {
            (true, true) | (false, false) => Self::new_positive(abs_value),
            (false, true) | (true, false) => Self::new_negative(abs_value),
        }
    }
}

impl std::ops::MulAssign for Integer {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl std::ops::Div for Integer {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let abs_value = self.value / rhs.value;
        match (self.negative, rhs.negative) {
            (true, true) | (false, false) => Self::new_positive(abs_value),
            (false, true) | (true, false) => Self::new_negative(abs_value),
        }
    }
}
impl std::ops::DivAssign for Integer {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl std::ops::Add for Integer {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self.negative, rhs.negative) {
            (false, false) => Self::new_positive(self.value + rhs.value),
            (true, true) => Self::new_negative(self.value + rhs.value),
            (false, true) => {
                if self.value >= rhs.value {
                    Self::new_positive(self.value - rhs.value)
                } else {
                    Self::new_negative(rhs.value - self.value)
                }
            }
            (true, false) => {
                if self.value >= rhs.value {
                    Self::new_negative(self.value - rhs.value)
                } else {
                    Self::new_positive(rhs.value - self.value)
                }
            }
        }
    }
}
impl std::ops::AddAssign for Integer {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for Integer {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self {
        self + rhs.invert_sign()
    }
}
impl std::ops::SubAssign for Integer {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn sub_assign(&mut self, rhs: Self) {
        *self += rhs.invert_sign();
    }
}

impl std::cmp::PartialOrd for Integer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //implementing this in terms of cmp should be fine
        //but just in case value.partial_cmp returns a None..
        if self.is_negative() && other.is_positive() {
            Some(Ordering::Less)
        } else if self.is_positive() && other.is_negative() {
            Some(Ordering::Greater)
        } else {
            self.value.partial_cmp(&other.value)
        }
    }
}

impl std::cmp::Ord for Integer {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_negative() && other.is_positive() {
            Ordering::Less
        } else if self.is_positive() && other.is_negative() {
            Ordering::Greater
        } else {
            self.value.cmp(&other.value)
        }
    }
}

/// Serializes as a string
impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as string
impl<'de> Deserialize<'de> for Integer {
    fn deserialize<D>(deserializer: D) -> Result<Integer, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IntegerVisitor)
    }
}

struct IntegerVisitor;

impl<'de> de::Visitor<'de> for IntegerVisitor {
    type Value = Integer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Integer::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{}': {}", v, e))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Integer;
    use std::str::FromStr;

    #[test]
    fn integer_default() {
        assert_eq!(Integer::ZERO, Integer::default());
    }

    #[test]
    fn integer_serialize() {
        let a = Integer::from(300u128);
        let b = Integer::from(7u128);
        let res = a / b;

        assert_eq!(serde_json::to_value(&res).unwrap(), "42");
        assert_eq!(serde_json::from_str::<Integer>("\"42\"").unwrap(), res);

        let res = res.invert_sign();

        assert_eq!(serde_json::to_value(&res).unwrap(), "-42");
        assert_eq!(serde_json::from_str::<Integer>("\"-42\"").unwrap(), res);
    }

    #[test]
    fn integer_arithmetic() {
        let a = Integer::from(300u128);
        let b = Integer::from(7u128);

        assert_eq!((a + b).to_string(), "307");
        assert_eq!((a - b).to_string(), "293");
        assert_eq!((b - a).to_string(), "-293");
        // #[cfg(feature = "floats")]
        // assert_eq!((a * b).to_f64_lossy(None), 2100.0);
        assert_eq!((a * b).to_string(), "2100");
        assert_eq!((a / b).to_string(), "42");
        // #[cfg(feature = "floats")]
        // assert_eq!((a / b).to_f64_lossy(Some(2)), 42.86);

        let a = a.invert_sign();
        let b = b.invert_sign();
        assert_eq!((a + b).to_string(), "-307");
        assert_eq!((a - b).to_string(), "-293");
        assert_eq!((b - a).to_string(), "293");
        assert_eq!((a * b).to_string(), "2100");
        assert_eq!((a / b).to_string(), "42");

        let a = a.invert_sign();
        assert_eq!((a + b).to_string(), "293");
        assert_eq!((a - b).to_string(), "307");
        assert_eq!((b - a).to_string(), "-307");
        assert_eq!((a * b).to_string(), "-2100");
        assert_eq!((a / b).to_string(), "-42");
    }

    #[test]
    fn integer_cmp() {
        let a = Integer::from_str("42").unwrap();
        let b = Integer::from_str("007").unwrap();

        assert!(a > b);
        assert!(!(a == b));

        let a = Integer::from_str("42").unwrap();
        let b = Integer::from_str("42").unwrap();

        assert!(!(a > b));
        assert!(a >= b);
        assert!(a == b);

        let a = Integer::from_str("42").unwrap();
        let b = Integer::from_str("-42").unwrap();

        assert!(a > b);
        assert!(!(a == b));
    }

    #[test]
    fn zero_str() {
        let mut a = Integer::from_str("0").unwrap();
        a.negative = true;
        assert_eq!(a.to_string(), "0");

        let a = Integer::from_str("-0").unwrap();
        assert_eq!(a.to_string(), "0");
    }
}
