use arbitrary::{Arbitrary, Unstructured};
use yoke::Yoke;

/// The control plane of chaos mode.
/// Please see the [crate-level documentation](crate).
pub struct ControlPlane {
    // The cart is the vector of pseudo-random data generated by a fuzzer.
    // The yoke is the slice of unused bytes in that vector.
    data: Yoke<&'static [u8], Vec<u8>>,
}

impl Default for ControlPlane {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Clone for ControlPlane {
    fn clone(&self) -> Self {
        Self::new(self.data.get().to_vec())
    }
}

impl std::fmt::Debug for ControlPlane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ControlPlane")
            .field("unused_bytes", &self.data.get())
            .field("backing_vector", self.data.backing_cart())
            .finish()
    }
}

impl Arbitrary<'_> for ControlPlane {
    fn arbitrary<'a>(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary().map(Self::new)
    }
}

impl ControlPlane {
    fn new(data: Vec<u8>) -> Self {
        let data = Yoke::attach_to_cart(data, |slice| slice);
        Self { data }
    }

    /// Returns a pseudo-random boolean if the control plane was constructed
    /// with `arbitrary`.
    ///
    /// The default value `false` will always be returned if the
    /// pseudo-random data is exhausted or the control plane was constructed
    /// with `default`.
    pub fn get_decision(&mut self) -> bool {
        let mut res = false;
        let data = std::mem::take(self).data.map_project(|unused_bytes, _| {
            let mut u = Unstructured::new(unused_bytes);
            res = u.arbitrary().unwrap_or_default();
            u.take_rest()
        });
        *self = Self { data };
        res
    }

    /// Shuffles the items in the slice into a pseudo-random permutation if
    /// the control plane was constructed with `arbitrary`.
    ///
    /// The default operation, to leave the slice unchanged, will always be
    /// performed if the pseudo-random data is exhausted or the control
    /// plane was constructed with `default`.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let data = std::mem::take(self).data.map_project(|unused_bytes, _| {
            let mut u = Unstructured::new(unused_bytes);

            // adapted from:
            // https://docs.rs/arbitrary/1.3.0/arbitrary/struct.Unstructured.html#examples-1
            let mut to_permute = &mut slice[..];

            while to_permute.len() > 1 {
                if let Ok(idx) = u.choose_index(to_permute.len()) {
                    to_permute.swap(0, idx);
                    to_permute = &mut to_permute[1..];
                } else {
                    break;
                }
            }
            u.take_rest()
        });
        *self = Self { data };
    }
}
