use std::env;

use config::{Config, ConfigError};
use serde::Deserialize;

use crate::configuration::application::ApplicationSettings;
use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::configuration::postgres::PostgresSettings;
use crate::configuration::qdrant::QdrantSettings;

enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _ => Err(format!(
                "{} is not a supported environment. Use either 'local' or 'production'",
                s
            )),
        }
    }
}

#[derive(Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub postgres: PostgresSettings,
    pub qdrant: QdrantSettings,
    pub itools: ItoolsSettings,
    pub common: CommonSettings,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let current_dir = env::current_dir().expect("Failed to determine the current directory");
    let configuration_dir = current_dir.join("configuration");

    // 通过`APP_ENVIRONMENT`环境变量检查运行时环境, 如果没有指定, 则默认是local, 也就是本地开发环境
    let environment: Environment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let environment_filename = format!("{}.yaml", environment.as_str());

    Config::builder()
        // 从配置文件中读取设置
        .add_source(config::File::from(configuration_dir.join("base.yaml")))
        // 后添加的源会覆盖先前存在的设置
        .add_source(config::File::from(
            configuration_dir.join(environment_filename),
        ))
        // 从环境变量中添加设置(前缀为APP, 将'__'作为分隔符)
        // 例如, 通过'APP__APPLICATION_PORT=5001'可以设置Settings.application.port=5001
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("__")
                .separator("_"),
        )
        .build()?
        .try_deserialize::<_>()
}
