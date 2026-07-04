use core::ops::Not;

#[cfg(feature = "bitint")]
use crate::bitint::u1;

/// Negated boolean: stores the negation of its logical value.
///
/// `NBool(false)` represents `true`, and `NBool(true)` represents `false`.
/// The inverted storage is useful in packed bit-fields where zero-initialized memory
/// should default to `true` (e.g. "enabled" flags).
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct NBool(bool);
impl NBool {
    /// Constructs an `NBool` from the raw `u1` bit value (`1` = false, `0` = true).
    #[cfg(feature = "bitint")]
    pub const fn new_with_raw_value(v: u1) -> Self {
        Self(v.value() != 0)
    }
    /// Returns the raw `u1` bit representation (`0` = true, `1` = false).
    #[cfg(feature = "bitint")]
    pub const fn raw_value(self) -> u1 {
        unsafe { u1::new_unchecked(self.0 as u8) }
    }
    /// Returns the logical boolean value (negation of the stored bit).
    pub const fn value(self) -> bool {
        !self.0
    }
    /// Constructs an `NBool` from a logical boolean value.
    pub const fn new(v: bool) -> Self {
        Self(!v)
    }
}

impl Not for NBool {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.value()
    }
}

impl From<bool> for NBool {
    fn from(value: bool) -> Self {
        Self(!value)
    }
}

impl From<NBool> for bool {
    fn from(value: NBool) -> Self {
        !value.0
    }
}
