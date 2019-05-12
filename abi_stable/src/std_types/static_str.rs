use std::{
    borrow::Borrow,
    fmt::{self, Display},
    marker::PhantomData,
    mem,
    ops::Deref,
};

use crate::std_types::RStr;

pub use self::inner::StaticStr;

// A type-level assertion that &[u8] is 2 usizes large.
type Assertions = [u8; {
    const USIZE_SIZE: usize = mem::size_of::<usize>();
    const SAME_SIZE: bool = 2 * USIZE_SIZE == mem::size_of::<&'static str>();
    const SAME_ALIGN: bool = mem::align_of::<[usize; 2]>() == mem::align_of::<&'static str>();
    ((SAME_SIZE & SAME_ALIGN) as usize) - 1
}];

mod inner {
    use super::*;

    /// Wrapper type around `&'static str` as a workaround for the
    /// non-stable-constness of str::len.
    #[derive(Copy, Clone, StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[repr(C)]
    pub struct StaticStr {
        #[sabi(unsafe_opaque_field)]
        s: &'static str,
        #[sabi(unsafe_opaque_field)]
        conversion: RStrFromStaticStr,
        _private_initializer: PhantomData<Assertions>,
    }

    impl StaticStr {
        /// Creates a StaticStr from a `&'static str`
        #[inline]
        pub const fn new(s: &'static str) -> Self {
            StaticStr {
                s,
                conversion: RStrFromStaticStr::NEW,
                _private_initializer: PhantomData,
            }
        }
        /// Gets the `&'static str` back.
        #[inline]
        pub fn as_str(&self) -> &'static str {
            self.as_rstr().into()
        }
        /// Converts the internal `&'static str` into a `RStr<'static>`.
        #[inline]
        pub fn as_rstr(&self) -> RStr<'static> {
            let s = (&self.s) as *const &'static str as *const [usize; 2];
            unsafe { (self.conversion.conversion)(s) }
        }
    }

    #[derive(Copy, Clone)]
    #[repr(transparent)]
    pub struct RStrFromStaticStr {
        conversion: unsafe extern "C" fn(*const [usize; 2]) -> RStr<'static>,
    }

    impl RStrFromStaticStr {
        const NEW: Self = {
            RStrFromStaticStr {
                conversion: str_conversion,
            }
        };
    }

}

#[doc(hidden)]
/*
This is the error that not marking this as pub causes:

error: internal compiler error: src/librustc_mir/monomorphize/collector.rs:747: Cannot create local mono-item for DefId(16/0:21 ~ abi_stable[af95]::static_str[0]::str_conversion[0])

thread 'rustc' panicked at 'Box<Any>', src/librustc_errors/lib.rs:588:9
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
stack backtrace:
   0: std::sys::unix::backtrace::tracing::imp::unwind_backtrace
             at src/libstd/sys/unix/backtrace/tracing/gcc_s.rs:39
   1: std::sys_common::backtrace::_print
             at src/libstd/sys_common/backtrace.rs:70
   2: std::panicking::default_hook::{{closure}}
             at src/libstd/sys_common/backtrace.rs:58
             at src/libstd/panicking.rs:200
   3: std::panicking::default_hook
             at src/libstd/panicking.rs:215
   4: rustc::util::common::panic_hook
   5: core::ops::function::Fn::call
   6: proc_macro::bridge::client::<impl proc_macro::bridge::Bridge<'_>>::enter::{{closure}}::{{closure}}
             at src/libproc_macro/bridge/client.rs:301
   7: std::panicking::rust_panic_with_hook
             at src/libstd/panicking.rs:482
   8: std::panicking::begin_panic
   9: rustc_errors::Handler::bug
  10: rustc::util::bug::opt_span_bug_fmt::{{closure}}
  11: rustc::ty::context::tls::with_opt::{{closure}}
  12: rustc::ty::context::tls::with_context_opt
  13: rustc::ty::context::tls::with_opt
  14: rustc::util::bug::opt_span_bug_fmt
  15: rustc::util::bug::bug_fmt
  16: rustc_mir::monomorphize::collector::should_monomorphize_locally
  17: rustc_mir::monomorphize::collector::collect_miri
  18: rustc_mir::monomorphize::collector::collect_miri
  19: rustc_mir::monomorphize::collector::collect_miri
  20: rustc_mir::monomorphize::collector::collect_const
  21: <rustc_mir::monomorphize::collector::MirNeighborCollector<'a, 'tcx> as rustc::mir::visit::Visitor<'tcx>>::visit_const
  22: <rustc_mir::monomorphize::collector::MirNeighborCollector<'a, 'tcx> as rustc::mir::visit::Visitor<'tcx>>::visit_rvalue
  23: rustc_mir::monomorphize::collector::collect_items_rec
  24: rustc_mir::monomorphize::collector::collect_items_rec
  25: rustc_mir::monomorphize::collector::collect_items_rec
  26: rustc_mir::monomorphize::collector::collect_items_rec
  27: rustc_mir::monomorphize::collector::collect_items_rec
  28: rustc_mir::monomorphize::collector::collect_items_rec
  29: rustc_mir::monomorphize::collector::collect_items_rec
  30: rustc_mir::monomorphize::collector::collect_items_rec
  31: rustc_mir::monomorphize::collector::collect_crate_mono_items::{{closure}}
  32: rustc::util::common::time
  33: rustc_mir::monomorphize::collector::collect_crate_mono_items
  34: rustc::util::common::time
  35: rustc_mir::monomorphize::partitioning::collect_and_partition_mono_items
  36: rustc::ty::query::__query_compute::collect_and_partition_mono_items
  37: rustc::ty::query::<impl rustc::ty::query::config::QueryAccessors<'tcx> for rustc::ty::query::queries::collect_and_partition_mono_items<'tcx>>::compute
  38: rustc::dep_graph::graph::DepGraph::with_task_impl
  39: rustc::ty::query::plumbing::<impl rustc::ty::context::TyCtxt<'a, 'gcx, 'tcx>>::try_get_with
  40: rustc_codegen_ssa::base::codegen_crate
  41: <rustc_codegen_llvm::LlvmCodegenBackend as rustc_codegen_utils::codegen_backend::CodegenBackend>::codegen_crate
  42: rustc::util::common::time
  43: rustc_driver::driver::phase_4_codegen
  44: rustc_driver::driver::compile_input::{{closure}}
  45: <std::thread::local::LocalKey<T>>::with
  46: rustc::ty::context::TyCtxt::create_and_enter
  47: rustc_driver::driver::compile_input
  48: rustc_driver::run_compiler_with_pool
  49: <scoped_tls::ScopedKey<T>>::set
  50: rustc_driver::run_compiler
  51: <scoped_tls::ScopedKey<T>>::set
query stack during panic:
#0 [collect_and_partition_mono_items] collect_and_partition_mono_items
end of query stack
error: aborting due to previous error


note: the compiler unexpectedly panicked. this is a bug.

note: we would appreciate a bug report: https://github.com/rust-lang/rust/blob/master/CONTRIBUTING.md#bug-reports

note: rustc 1.33.0 (2aa4c46cf 2019-02-28) running on i686-unknown-linux-gnu

note: compiler flags: -C debuginfo=2 -C incremental --crate-type bin

note: some of the compiler flags provided by cargo are hidden

error: Could not compile `abi_stable`.


*/
pub unsafe extern "C" fn str_conversion(s: *const [usize; 2]) -> RStr<'static> {
    let str_: &'static str = *(s as *const &'static str);
    RStr::from(str_)
}

impl Deref for StaticStr {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for StaticStr{
    fn borrow(&self)->&str{
        self
    }
}

impl AsRef<str> for StaticStr{
    fn as_ref(&self)->&str{
        self
    }
}

impl AsRef<[u8]> for StaticStr{
    fn as_ref(&self)->&[u8]{
        self.as_bytes()
    }
}

impl Display for StaticStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

shared_impls! {
    mod=slice_impls
    new_type=StaticStr[][],
    original_type=str,
}
