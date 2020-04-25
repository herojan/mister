use std::collections::HashMap;
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mister",
    about = "Creates a directory for each microservice_standard_deployment process in a CDP delivery.yaml"
)]
pub struct Opt {
    #[structopt(
        short = "d",
        long = "delivery-yaml",
        parse(from_os_str),
        default_value = "delivery.yaml"
    )]
    pub delivery_yaml: PathBuf,
    #[structopt(
        short = "o",
        long = "output-dir",
        parse(from_os_str),
        default_value = "mister"
    )]
    pub output_dir: PathBuf,

    #[structopt(
        short = "b",
        long = "cdp-build-version",
        default_value = "#{CDP_BUILD_VERSION}"
    )]
    pub cdp_build_version: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplyResources {
    pub env: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub apply_permanent_resources: Option<ApplyResources>,
    pub apply_manifests: Option<ApplyResources>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PipelineStep {
    pub id: String,
    pub process: Option<String>,
    pub config: Option<DeploymentConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Delivery {
    pub version: String,
    pub pipeline: Vec<PipelineStep>,
}
