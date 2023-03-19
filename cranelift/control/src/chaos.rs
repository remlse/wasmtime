use std::sync::{Arc, Mutex};

use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug)]
struct ControlPlaneData {
    /// This is the primary vector holding the bytes received from
    /// the fuzzer. In order to avoid lifetime proliferation, new
    /// [Unstructured] values with references into this vector are
    /// created for every control plane API call. `primary` is then
    /// updated to exclude the bytes consumed by [Unstructured],
    /// ensuring no bytes are reused across control plane API calls.
    ///
    /// [Unstructured]: arbitrary::unstructured::Unstructured
    primary: Mutex<Vec<u8>>,
    /// This is a temporary buffer used for updating the primary one
    /// without needing additional heap allocations.
    tmp: Mutex<Vec<u8>>,
}

impl ControlPlaneData {
    fn new(data: Vec<u8>) -> Self {
        Self {
            primary: Mutex::new(data.clone()),
            tmp: Mutex::new(data),
        }
    }
}

/// The control plane of chaos mode.
/// Please see the [crate-level documentation](crate).
///
/// **Clone liberally!** The control plane is reference counted.
#[derive(Debug, Clone)]
pub struct ControlPlane {
    data: Arc<ControlPlaneData>,
    is_todo: bool,
}

impl ControlPlane {
    fn new(data: Vec<u8>, is_todo: bool) -> Self {
        Self {
            data: Arc::new(ControlPlaneData::new(data)),
            is_todo,
        }
    }

    /// This is a zero-sized dummy for use during any builds without the
    /// feature `chaos` enabled, especially release builds. It has no
    /// functionality, so the programmer is prevented from using it in any
    /// way in release builds, which could degrade performance.
    ///
    /// This should not be used on code paths that may execute while the
    /// feature `chaos` is enabled. That would break the assumption that
    /// [ControlPlane] is a singleton, responsible for centrally managing
    /// the pseudo-randomness injected at runtimme.
    ///
    /// Use [todo](ControlPlane::todo) instead, for stubbing out code paths
    /// you don't expect to be reached (yet) during chaos mode fuzzing.
    ///
    /// # Panics
    ///
    /// Panics if it is called while the feature `chaos` is enabled.
    #[track_caller]
    pub fn noop() -> Self {
        panic!(
            "attempted to create a NOOP control plane \
            (while chaos mode was enabled)"
        );
    }

    /// This is the same as [noop](ControlPlane::noop) when the the feature
    /// `chaos` is *disabled*. When `chaos` is enabled, it returns a
    /// control plane that returns [Error::Todo] when
    /// [get_arbitrary](ControlPlane::get_arbitrary) is called.
    ///
    /// This may be used during development, in places which are (supposed
    /// to be) unreachable during fuzzing. Use of this function should be
    /// reduced as the chaos mode is introduced in more parts of the
    /// wasmtime codebase. Eventually, it should be deleted.
    pub fn todo() -> Self {
        Self::new(Vec::new(), true)
    }
}

impl<'a> Arbitrary<'a> for ControlPlane {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(u.arbitrary()?, false))
    }
    fn arbitrary_take_rest(u: Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(Arbitrary::arbitrary_take_rest(u)?, false))
    }
}

/// An enumeration of control plane API errors, mostly propagating
/// [arbitrary::Error].
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// No choices were provided to the Unstructured::choose call.
    EmptyChoose,
    /// There was not enough underlying data to fulfill some request for raw
    /// bytes.
    NotEnoughData,
    /// The input bytes were not of the right format.
    IncorrectFormat,
    /// The control plane API was accessed on a [ControlPlane::todo].
    Todo,
}

impl From<arbitrary::Error> for Error {
    fn from(value: arbitrary::Error) -> Self {
        // Force this match statement to be updated when arbitrary
        // introduces new error variants.
        #[deny(clippy::wildcard_enum_match_arm)]
        match value {
            arbitrary::Error::EmptyChoose => Error::EmptyChoose,
            arbitrary::Error::NotEnoughData => Error::NotEnoughData,
            arbitrary::Error::IncorrectFormat => Error::IncorrectFormat,
            _ => unreachable!("must propagate all error variants"),
        }
    }
}

impl ControlPlane {
    /// Request an arbitrary boolean from the control plane.
    ///
    /// # Errors
    ///
    /// - Errors from an underlying call to [arbitrary] will be
    ///   propagated as-is.
    /// - Calling this function on a control plane received from a call to
    ///   [todo] will return an [Error::Todo].
    ///
    /// [arbitrary]: arbitrary::Arbitrary::arbitrary
    /// [todo]: ControlPlane::todo
    pub fn get_arbitrary_bool(&self) -> Result<bool, Error> {
        if self.is_todo {
            return Err(Error::Todo);
        }

        let value = {
            let primary = self.data.primary.lock().expect("poisoned");
            let mut u = Unstructured::new(&primary);
            let value = u.arbitrary().unwrap();

            let rest = u.take_rest();

            // store remaining bytes in tmp
            let mut tmp = self.data.tmp.lock().expect("poisoned");
            tmp.resize(rest.len(), 0);
            tmp.copy_from_slice(rest);

            value
        };

        // update primary with remaining bytes
        let mut primary = self.data.primary.lock().expect("poisoned");
        let mut tmp = self.data.tmp.lock().expect("poisoned");
        std::mem::swap::<Vec<_>>(primary.as_mut(), tmp.as_mut());

        Ok(value)
    }
}
