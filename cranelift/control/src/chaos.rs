use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug, Default, Arbitrary)]
struct ControlPlaneData {
    bools: Vec<bool>,
}

/// The control plane of chaos mode.
/// Please see the [crate-level documentation](crate).
///
/// **Clone liberally!** The control plane is reference counted.
#[derive(Debug, Clone)]
pub struct ControlPlane {
    data: Arc<Mutex<ControlPlaneData>>,
    is_todo: bool,
}

impl ControlPlane {
    fn new(data: ControlPlaneData, is_todo: bool) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
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
        Self::new(ControlPlaneData::default(), true)
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

/// An enumeration of control plane API errors.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// There was not enough underlying data to fulfill some request for raw
    /// bytes.
    NotEnoughData,
    /// The control plane API was accessed on a [ControlPlane::todo].
    Todo,
}

impl ControlPlane {
    /// Request an arbitrary bool from the control plane.
    ///
    /// # Errors
    ///
    /// - There was not enough underlying data to fulfill some request for
    ///   raw bytes: [Error::NotEnoughData].
    /// - Calling this function on a control plane received from a call to
    ///   [todo] will return an [Error::Todo].
    ///
    /// [arbitrary]: arbitrary::Arbitrary::arbitrary
    /// [todo]: ControlPlane::todo
    pub fn get_arbitrary_bool(&self) -> Result<bool, Error> {
        if self.is_todo {
            return Err(Error::Todo);
        }
        self.data
            .lock()
            .expect("poisoned ControlPlaneData mutex")
            .bools
            .pop()
            .ok_or(Error::NotEnoughData)
    }
}
