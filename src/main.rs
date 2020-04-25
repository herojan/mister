extern crate exitfailure;
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate structopt;

use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, read_to_string};
use std::fs::{File, ReadDir};
use std::io::BufReader;
use std::path::PathBuf;

use exitfailure::ExitFailure;
use failure::err_msg;
use failure::Error;
use failure::ResultExt;
use structopt::StructOpt;

use model::Opt;
use model::{Delivery, DeploymentConfig};

mod model;

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();
    let output_dir_opt = opt.output_dir;
    let delivery_yaml_opt = opt.delivery_yaml;
    let cdp_build_version_opt = opt.cdp_build_version;

    let delivery_yaml = File::open(&delivery_yaml_opt)?;
    let delivery_yaml = BufReader::new(&delivery_yaml);
    let delivery_yaml: Delivery = serde_yaml::from_reader(delivery_yaml)?;

    for step in delivery_yaml.pipeline {
        let step_id = step.id;
        let config = step
            .process
            .filter(|p| {
                p == "microservice_standard_deployment" || p == "microservice_standard_test"
            })
            .and(step.config);

        if let Some(config) = config {
            let env = parse_env(config, &cdp_build_version_opt)?;
            let output_dir = &output_dir_opt;

            rewrite_resources(&output_dir.join(step_id), env)?;
        } else {
            println!("Skipping pipeline step with id '{}'", step_id)
        }
    }

    Ok(())
}

fn parse_env(
    config: DeploymentConfig,
    cdp_build_version: &str,
) -> Result<HashMap<String, String>, ExitFailure> {
    let mut env = config
        .apply_manifests
        .xor(config.apply_permanent_resources)
        .ok_or(err_msg("deployment step with id '{}' must contain either 'apply_permanent_resources' or 'apply_manifests'"))?.env;

    for val in env.values_mut() {
        *val = val.replace("#{CDP_BUILD_VERSION}", cdp_build_version);
    }
    Ok(env)
}

fn rewrite_resources(output_dir: &PathBuf, env: HashMap<String, String>) -> Result<(), Error> {
    if !output_dir.exists() {
        create_dir_all(&output_dir).context(format!(
            "Could not create directory '{}'",
            output_dir.display()
        ))?;
    }
    for resource in find_deploy_resources_dir(&env)? {
        let resource = resource?;
        let path = resource.path();
        if path.is_file() {
            let template = read_to_string(&path)?;
            let template = mustache::compile_str(&template)?;
            let path_buf = output_dir.join(resource.file_name());
            let output_file = &mut File::create(path_buf.as_path())?;

            template.render(output_file, &env)?;
            println!("Outputting resource: '{:?}'", path_buf.as_path())
        }
    }

    Ok(())
}

fn find_deploy_resources_dir(env: &HashMap<String, String>) -> Result<ReadDir, Error> {
    let deploy_path = env
        .get("DEPLOYMENT_PATH")
        .map(|s| format!("{}/apply", s))
        .unwrap_or("deploy/apply".to_owned());

    let read_dir = read_dir(&deploy_path).context(format!(
        "DEPLOYMENT_PATH directory '{}' does not exist",
        deploy_path
    ))?;

    Ok(read_dir)
}
