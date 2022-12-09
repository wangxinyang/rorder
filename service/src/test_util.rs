use std::{ops::Deref, path::Path};

use abi::Config;
use sqlx_mock::TestPostgres;

pub struct TestConfig {
    #[allow(dead_code)]
    tps: TestPostgres,
    pub config: Config,
}

impl TestConfig {
    pub fn new(filename: impl AsRef<Path>) -> Self {
        let mut config = Config::from_file(filename).unwrap();
        let server_url = format!(
            "postgres://{}:{}@{}",
            &config.db.user, &config.db.password, &config.db.host
        );
        let tps = TestPostgres::new(server_url, Path::new("../migrations"));
        config.db.dbname = tps.dbname.clone();
        Self { tps, config }
    }
}

impl Deref for TestConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new("fixtures/config.yml")
    }
}
