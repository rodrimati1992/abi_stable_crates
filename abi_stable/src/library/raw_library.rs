use super::*;

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

/// A handle to any dynamically loaded library,
/// not necessarily ones that export abi_stable compatible modules.
pub struct RawLibrary {
    path: PathBuf,
    library: LibLoadingLibrary,
}

impl RawLibrary {
    /// Gets the full path a library would be loaded from,
    pub fn path_in_directory(directory: &Path, base_name: &str, suffix: LibrarySuffix) -> PathBuf {
        let formatted: String;

        let is_64_bits =
            cfg!(any(x86_64, powerpc64, aarch64)) || ::std::mem::size_of::<usize>() == 8;
        let bits = if is_64_bits { "64" } else { "32" };

        let maybe_suffixed_name = match suffix {
            LibrarySuffix::Suffix => {
                formatted = format!("{}-{}", base_name, bits);
                &*formatted
            }
            LibrarySuffix::NoSuffix => base_name,
        };

        let name = format!("{}{}{}", DLL_PREFIX, maybe_suffixed_name, DLL_SUFFIX);
        directory.join(name)
    }

    /// Loads the dynamic library at the `full_path` path.
    pub fn load_at(full_path: &Path) -> Result<Self, LibraryError> {
        // safety: not my problem if libraries have problematic static initializers
        match unsafe { LibLoadingLibrary::new(full_path) } {
            Ok(library) => Ok(Self {
                path: full_path.to_owned(),
                library,
            }),
            Err(err) => Err(LibraryError::OpenError {
                path: full_path.to_owned(),
                err: Box::new(err),
            }),
        }
    }

    /// Gets access to a static/function declared by the library.
    ///
    /// # Safety
    ///
    /// Passing a `T` of a type different than the compiled library declared is
    /// undefined behavior.
    ///
    ///
    ///
    pub unsafe fn get<T>(&self, symbol_name: &[u8]) -> Result<LLSymbol<'_, T>, LibraryError> {
        match unsafe { self.library.get::<T>(symbol_name) } {
            Ok(symbol) => Ok(symbol),
            Err(io) => {
                let symbol = symbol_name.to_owned();
                Err(LibraryError::GetSymbolError {
                    library: self.path.clone(),
                    symbol,
                    err: Box::new(io),
                })
            }
        }
    }
}
