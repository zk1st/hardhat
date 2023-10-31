mod node;

use tempfile::TempDir;

pub use node::NodeTestFixture;

/// Type used for tests that need to guarantee that the temporary cache directory exists for the lifetime of the test.
pub struct CacheDirTestFixture<TestDataT> {
    cache_dir: TempDir,
    pub test_data: TestDataT,
}

impl<TestDataT> CacheDirTestFixture<TestDataT> {
    /// Constructs an instance.
    pub fn new<FactoryFnT>(factory_fn: FactoryFnT) -> Self
    where
        FactoryFnT: FnOnce(&TempDir) -> TestDataT,
    {
        let cache_dir = TempDir::new().expect("should create temp dir");
        let test_data = factory_fn(&cache_dir);

        Self {
            cache_dir,
            test_data,
        }
    }

    /// Retrieves the cache directory.
    pub fn cache_dir(&self) -> &TempDir {
        &self.cache_dir
    }
}
