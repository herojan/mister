extern crate exitfailure;
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate structopt;

use exitfailure::ExitFailure;
use failure::err_msg;
use failure::Error;
use failure::ResultExt;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::fs::{create_dir_all, read_dir, read_to_string};
use std::io::BufReader;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mister",
    about = "Creates a directory for each microservice_standard_deployment process in a CDP delivery.yaml"
)]
struct Opt {
    #[structopt(
        short = "d",
        long = "delivery-yaml",
        parse(from_os_str),
        default_value = "delivery.yaml"
    )]
    delivery_yaml: PathBuf,
    #[structopt(
        short = "o",
        long = "output-dir",
        parse(from_os_str),
        default_value = "mister"
    )]
    output_dir: PathBuf,

    #[structopt(
    short = "b",
    long = "cdp-build-version",
    default_value = "#{CDP_BUILD_VERSION}"
    )]
    cdp_build_version: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ApplyResources {
    env: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DeploymentConfig {
    apply_permanent_resources: Option<ApplyResources>,

    apply_manifests: Option<ApplyResources>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Deployment {
    id: String,
    config: DeploymentConfig,
}

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();
    let output_dir_opt = opt.output_dir;
    let delivery_yaml_opt = opt.delivery_yaml;
    let cdp_build_version_opt = opt.cdp_build_version;

    let delivery_yaml = File::open(&delivery_yaml_opt)?;
    let delivery_yaml = BufReader::new(&delivery_yaml);
    let delivery_yaml: BTreeMap<String, Value> = serde_yaml::from_reader(delivery_yaml)?;

    let pipeline = delivery_yaml
        .get("pipeline")
        .ok_or(err_msg(format!(
            "Missing pipeline key in delivery yaml file '{:#?}'",
            delivery_yaml_opt
        )))?
        .as_sequence()
        .ok_or(err_msg(format!(
            "Pipeline must be an array in delivery yaml file '{:#?}'",
            delivery_yaml_opt
        )))?;

    let (pipeline, errors) = parse_pipeline_steps(pipeline);

    for error in errors {
        println!("{}", error);
    }

    for deployment in pipeline {
        rewrite_resources(&output_dir_opt, deployment, &cdp_build_version_opt)?;
    }

    Ok(())
}

fn rewrite_resources(output_dir: &PathBuf, deployment: Deployment, cdp_build_version: &str) -> Result<(), Error> {
    let config = deployment.config;
    let mut env = config
        .apply_permanent_resources
        .or(config.apply_manifests)
        .ok_or(err_msg("deployment config must contain either 'apply_permanent_resources' or 'apply_manifests'"))?
        .env;

    for val in env.values_mut() {
        *val = val.replace("#{CDP_BUILD_VERSION}", cdp_build_version);
    }
    let deploy_path = env
        .get("DEPLOYMENT_PATH")
        .map(String::as_ref)
        .unwrap_or("deploy/apply");

    let entries = read_dir(deploy_path).context(format!(
        "DEPLOYMENT_PATH directory '{}' does not exist",
        deploy_path
    ))?;

    let output_dir = output_dir.join(deployment.id.as_str());
    //    let output_path = Path::new(format!()deployment.id.as_str());
    if !output_dir.exists() {
        create_dir_all(&output_dir).context(format!(
            "Could not create directory '{}'",
            output_dir.display()
        ))?;
    }

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let template = read_to_string(&path)?;
            let template = mustache::compile_str(&template)?;
            let path_buf = output_dir.join(entry.file_name());
            let output_file = &mut File::create(path_buf.as_path())?;

            template.render(output_file, &env)?;
            println!("Outputting resource: '{:?}'", path_buf.as_path())
        }
    }

    Ok(())
}

fn parse_pipeline_steps(pipeline_values: &Vec<Value>) -> (Vec<Deployment>, Vec<serde_yaml::Error>) {
    let (pipeline, errors): (Vec<_>, Vec<_>) = pipeline_values
        .into_iter()
        .filter(|v| {
            let process_name = v.get("process").and_then(|v| v.as_str());

            process_name
                .filter(|&p| {
                    p == "microservice_standard_deployment" || p == "microservice_standard_test"
                })
                .is_some()
        })
        .map(|deployment| serde_yaml::from_value::<Deployment>(deployment.to_owned()))
        .partition(Result::is_ok);

    let pipeline = pipeline.into_iter().map(Result::unwrap).collect();
    let errors = errors.into_iter().map(Result::unwrap_err).collect();

    (pipeline, errors)
}
