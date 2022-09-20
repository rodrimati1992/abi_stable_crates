use super::*;

use crate::StableAbi;

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
};

///////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy, StableAbi)]
pub struct RSendError<T>(pub T);

impl<T> RSendError<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Debug for RSendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("RSendError{..}")
    }
}

impl<T> Display for RSendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Attempting to send on a disconnected channel")
    }
}

impl<T> ErrorTrait for RSendError<T> {}

impl_from_rust_repr! {
    impl[T] From<SendError<T>> for RSendError<T> {
        fn(this){
            RSendError(this.into_inner())
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<SendError<T>> for RSendError<T> {
        fn(this){
            SendError(this.into_inner())
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, StableAbi)]
pub struct RRecvError;

impl Debug for RRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("RRecvError{..}")
    }
}

impl Display for RRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Attempting to recv on a disconnected channel")
    }
}

impl ErrorTrait for RRecvError {}

impl_from_rust_repr! {
    impl From<RecvError> for RRecvError {
        fn(_this){
            RRecvError
        }
    }
}

impl_into_rust_repr! {
    impl Into<RecvError> for RRecvError {
        fn(_this){
            RecvError
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, StableAbi)]
pub enum RTrySendError<T> {
    Full(T),
    Disconnected(T),
}

impl<T> RTrySendError<T> {
    pub fn into_inner(self) -> T {
        match self {
            RTrySendError::Full(v) => v,
            RTrySendError::Disconnected(v) => v,
        }
    }
    pub fn is_full(&self) -> bool {
        matches!(self, RTrySendError::Full { .. })
    }
    pub fn is_disconnected(&self) -> bool {
        matches!(self, RTrySendError::Disconnected { .. })
    }
}

impl<T> Debug for RTrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            RTrySendError::Full { .. } => "Full{..}",
            RTrySendError::Disconnected { .. } => "Disconnected{..}",
        };
        f.pad(msg)
    }
}

impl<T> Display for RTrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            RTrySendError::Full { .. } => "Attempting to send on a full channel",
            RTrySendError::Disconnected { .. } => "Attempting to send on a disconnected channel",
        };
        f.pad(msg)
    }
}

impl<T> ErrorTrait for RTrySendError<T> {}

impl_from_rust_repr! {
    impl[T] From<TrySendError<T>> for RTrySendError<T> {
        fn(this){
            match this {
                TrySendError::Full(v)=>
                    RTrySendError::Full(v),
                TrySendError::Disconnected(v)=>
                    RTrySendError::Disconnected(v),
            }
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<TrySendError<T>> for RTrySendError<T> {
        fn(this){
            match this {
                RTrySendError::Full(v)=>TrySendError::Full(v),
                RTrySendError::Disconnected(v)=>TrySendError::Disconnected(v),
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, StableAbi)]
pub enum RTryRecvError {
    Empty,
    Disconnected,
}

impl RTryRecvError {
    pub fn is_empty(&self) -> bool {
        *self == RTryRecvError::Empty
    }

    pub fn is_disconnected(&self) -> bool {
        *self == RTryRecvError::Disconnected
    }
}

impl Display for RTryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            RTryRecvError::Empty { .. } => "Attempting to recv on an empty channel",
            RTryRecvError::Disconnected { .. } => "Attempting to recv on a disconnected channel",
        };
        f.pad(msg)
    }
}

impl ErrorTrait for RTryRecvError {}

impl_from_rust_repr! {
    impl From<TryRecvError> for RTryRecvError {
        fn(this){
            match this {
                TryRecvError::Empty=>
                    RTryRecvError::Empty,
                TryRecvError::Disconnected=>
                    RTryRecvError::Disconnected,
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<TryRecvError> for RTryRecvError {
        fn(this){
            match this {
                RTryRecvError::Empty=>TryRecvError::Empty,
                RTryRecvError::Disconnected=>TryRecvError::Disconnected,
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, StableAbi)]
pub enum RSendTimeoutError<T> {
    Timeout(T),
    Disconnected(T),
}

impl<T> RSendTimeoutError<T> {
    pub fn into_inner(self) -> T {
        match self {
            RSendTimeoutError::Timeout(v) => v,
            RSendTimeoutError::Disconnected(v) => v,
        }
    }
    pub fn is_timeout(&self) -> bool {
        matches!(self, RSendTimeoutError::Timeout { .. })
    }
    pub fn is_disconnected(&self) -> bool {
        matches!(self, RSendTimeoutError::Disconnected { .. })
    }
}

impl<T> Debug for RSendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            RSendTimeoutError::Timeout { .. } => "Timeout{..}",
            RSendTimeoutError::Disconnected { .. } => "Disconnected{..}",
        };
        f.pad(msg)
    }
}

impl<T> Display for RSendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match *self {
            RSendTimeoutError::Timeout { .. } => "Timed out while attempting to send on a channel",
            RSendTimeoutError::Disconnected { .. } => {
                "Attempting to send on a disconnected channel"
            }
        };
        f.pad(msg)
    }
}

impl<T> ErrorTrait for RSendTimeoutError<T> {}

impl_from_rust_repr! {
    impl[T] From<SendTimeoutError<T>> for RSendTimeoutError<T> {
        fn(this){
            match this {
                SendTimeoutError::Timeout(v)=>
                    RSendTimeoutError::Timeout(v),
                SendTimeoutError::Disconnected(v)=>
                    RSendTimeoutError::Disconnected(v),
            }
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<SendTimeoutError<T>> for RSendTimeoutError<T> {
        fn(this){
            match this {
                RSendTimeoutError::Timeout(v)=>
                    SendTimeoutError::Timeout(v),
                RSendTimeoutError::Disconnected(v)=>
                    SendTimeoutError::Disconnected(v),
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, StableAbi)]
pub enum RRecvTimeoutError {
    Timeout,
    Disconnected,
}

impl RRecvTimeoutError {
    pub fn is_timeout(&self) -> bool {
        matches!(self, RRecvTimeoutError::Timeout { .. })
    }
    pub fn is_disconnected(&self) -> bool {
        matches!(self, RRecvTimeoutError::Disconnected { .. })
    }
}

impl Display for RRecvTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            RRecvTimeoutError::Timeout { .. } => "Timed out while attempting to recv on a channel",
            RRecvTimeoutError::Disconnected { .. } => {
                "Attempting to recv on a disconnected channel"
            }
        };
        f.pad(msg)
    }
}

impl ErrorTrait for RRecvTimeoutError {}

impl_from_rust_repr! {
    impl From<RecvTimeoutError> for RRecvTimeoutError {
        fn(this){
            match this {
                RecvTimeoutError::Timeout=>
                    RRecvTimeoutError::Timeout,
                RecvTimeoutError::Disconnected=>
                    RRecvTimeoutError::Disconnected,
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<RecvTimeoutError> for RRecvTimeoutError {
        fn(this){
            match this {
                RRecvTimeoutError::Timeout=>
                    RecvTimeoutError::Timeout,
                RRecvTimeoutError::Disconnected=>
                    RecvTimeoutError::Disconnected,
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
