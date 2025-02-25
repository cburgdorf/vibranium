use crate::config;
use crate::project_generator;

use super::error::DeploymentTrackingError;

use config::Config;
use project_generator::VIBRANIUM_PROJECT_DIRECTORY;
use std::io::Write;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use sha3::{Digest, Sha3_256};
use toml;
use toml_query::insert::TomlValueInsertExt;
use toml_query::set::TomlValueSetExt;
use toml_query::read::TomlValueReadExt;
use web3::types::{H256, Address};

pub const TRACKING_FILE: &str = "tracking.toml";

pub type SmartContractTrackingData = HashMap<String, SmartContractTrackingDataEntry>;
type TrackingData = HashMap<String, SmartContractTrackingData>;

#[derive(Serialize, Deserialize, Debug)]
pub struct SmartContractTrackingDataEntry {
  pub name: String,
  pub address: Address,
}

pub struct DeploymentTracker<'a> {
  config: &'a Config,
}

impl<'a> DeploymentTracker<'a> {
  pub fn new(config: &'a Config) -> DeploymentTracker<'a> {
    DeploymentTracker {
      config,
    }
  }

  pub fn database_exists(&self) -> bool {
    self.get_tracking_file().exists()
  }

  pub fn create_database(&self) -> Result<(), DeploymentTrackingError> {
    let _ = fs::File::create(&self.get_tracking_file())?;
    Ok(())
  }

  pub fn track(&self, block_hash: H256, name: String, byte_code: String, args: &Vec<String>, address: Address) -> Result<(), DeploymentTrackingError> {

    let block_hash = create_block_hash(&block_hash);
    let smart_contract_hash = create_smart_contract_hash(&name, &byte_code, &args);
    let query = format!("{}.{}", &block_hash, &smart_contract_hash);

    let smart_contract_tracking_data = SmartContractTrackingDataEntry { name, address, };

    let mut tracking_data = self.try_from_tracking_file()?;
    let chain_tracking_data = tracking_data.read(&block_hash)?;
    let new_tracking_data = toml::Value::try_from(smart_contract_tracking_data)?;
    
    match chain_tracking_data {
      None => tracking_data.insert(&query, new_tracking_data).map_err(DeploymentTrackingError::Insertion)?,
      Some(_) => tracking_data.set(&query, new_tracking_data).map_err(DeploymentTrackingError::Insertion)?,
    };

    self.write(tracking_data)
  }

  pub fn get_smart_contract_tracking_data(&self, block_hash: &H256, name: &str, byte_code: &str, args: &Vec<String>) -> Result<Option<SmartContractTrackingDataEntry>, DeploymentTrackingError> {
    let block_hash = create_block_hash(&block_hash);
    let smart_contract_hash = create_smart_contract_hash(&name, &byte_code, &args);
    let tracking_data = self.try_from_tracking_file()?;
    let contract_data = tracking_data.read(&format!("{}.{}", &block_hash, &smart_contract_hash))?;

    if let Some(contract_data) = contract_data {
      Ok(Some(contract_data.to_owned().try_into::<SmartContractTrackingDataEntry>()?))
    } else {
      Ok(None)
    }
  }

  pub fn get_all_smart_contract_tracking_data(&self, block_hash: &H256) -> Result<Option<SmartContractTrackingData>, DeploymentTrackingError> {
    let block_hash = create_block_hash(&block_hash);
    match self.try_from_tracking_file() {
      Err(_) => Ok(None),
      Ok(tracking_data) => {
        let contract_data = tracking_data.read(&format!("{}", &block_hash))?;
        if let Some(contract_data) = contract_data {
          Ok(Some(contract_data.to_owned().try_into::<SmartContractTrackingData>()?))
        } else {
          Ok(None)
        }
      }
    }
  }

  fn write(&self, toml: toml::Value) -> Result<(), DeploymentTrackingError> {
    let tracking_data = toml::to_string(&toml)?;
    let mut tracking_file= fs::File::create(&self.get_tracking_file())?;
    tracking_file.write_all(tracking_data.as_bytes()).map_err(|err| DeploymentTrackingError::Other(err.to_string()))
  }

  fn try_from_tracking_file(&self) -> Result<toml::Value, DeploymentTrackingError> {
    if !self.database_exists() {
      return Err(DeploymentTrackingError::DatabaseNotFound);
    }
    let tracking_data = fs::read_to_string(self.get_tracking_file())?;
    let toml: TrackingData = toml::from_str(&tracking_data)?;
    toml::Value::try_from(toml).map_err(DeploymentTrackingError::Serialization)
  }

  fn get_tracking_file(&self) -> PathBuf {
    let vibranium_dir = self.config.project_path.join(VIBRANIUM_PROJECT_DIRECTORY);
    vibranium_dir.join(TRACKING_FILE)
  }
}

fn create_block_hash(block_hash: &H256) -> String {
  format!("0x{:x}", Sha3_256::digest(block_hash.as_bytes()))
}

fn create_smart_contract_hash(name: &str, byte_code: &str, args: &Vec<String>) -> String {
  let mut hasher = Sha3_256::new();

  hasher.input(name.as_bytes());
  hasher.input(byte_code.as_bytes());
  hasher.input(args.join("").as_bytes());

  format!("0x{:x}", hasher.result())
}
