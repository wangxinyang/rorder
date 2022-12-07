use abi::Config;
use anyhow::Ok;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let filename = "./fixtures/config.yaml";
    let _config = Config::from_file(filename)?;
    Ok(())
}
