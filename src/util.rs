use errors::*;
use itertools::Itertools;
use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait HandyRwLock<T> {
    fn wl(&self) -> RwLockWriteGuard<T>;
    fn rl(&self) -> RwLockReadGuard<T>;
}

impl<T> HandyRwLock<T> for RwLock<T> {
    #[inline]
    fn wl(&self) -> RwLockWriteGuard<T> {
        self.write().unwrap()
    }

    #[inline]
    fn rl(&self) -> RwLockReadGuard<T> {
        self.read().unwrap()
    }
}

pub fn create_dir<P: AsRef<Path>>(p: P) -> Result<()> {
    DirBuilder::new().recursive(true).create(p)?;
    Ok(())
}

pub fn read_with_ext_and_sort<P: AsRef<Path>>(
    dir: P,
    ext: &str,
    desc: bool,
) -> Result<Vec<PathBuf>> {
    Ok(dir
        .as_ref()
        .read_dir()?
        .filter_map(|dir_entry_r| {
            if let Ok(dir_entry) = dir_entry_r {
                if let Ok(metadata) = dir_entry.metadata() {
                    if metadata.is_file() {
                        if let Some(extension) = dir_entry.path().extension() {
                            if extension == ext {
                                return Some(dir_entry.path());
                            }
                        }
                    }
                }
            }
            None
        })
        .sorted_by(|first, second| {
            if desc {
                first.cmp(second).reverse()
            } else {
                first.cmp(second)
            }
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_with_ext_test() {
        println!(
            "{:?}",
            read_with_ext_and_sort("./file_tests/", "snap", false)
        );
    }
}
