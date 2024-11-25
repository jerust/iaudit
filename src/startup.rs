use std::net::TcpListener;
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::web::{Data, JsonConfig};
use actix_web::{error, App, HttpResponse, HttpServer};
use qdrant_client::Qdrant;
use sqlx::PgPool;
use tokio::sync::Mutex;
use tracing_actix_web::TracingLogger;

use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::configuration::setting::Settings;

pub struct Application {
    server: Server,
    port: u16,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let addr = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(addr)?;
        let port = listener.local_addr()?.port();

        let pgpool = configuration.postgres.get_postgres_connection_pool();

        let qdrant = configuration
            .qdrant
            .get_qdrant_client()
            .expect("构建向量数据库客户端失败");

        // 对qdrant客户端健康状况进行检查, 如果有异常就直接退出应用程序
        qdrant.health_check().await.expect("向量数据库健康检查异常");

        let itools = configuration.itools;

        let common = configuration.common;

        let server = run(listener, pgpool, qdrant, itools, common)?;

        Ok(Self { server, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

// 设置请求JSON的最大值为10M
const MAX_JSON_BYTES: usize = 10 * 1 << 20;

fn build_json_configuration() -> JsonConfig {
    JsonConfig::default()
        .limit(MAX_JSON_BYTES)
        .error_handler(|err, _req| {
            error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
        })
}

pub fn run(
    listener: TcpListener,
    pgpool: PgPool,
    qdrant: Qdrant,
    itools: ItoolsSettings,
    common: CommonSettings,
) -> Result<Server, std::io::Error> {
    let server = HttpServer::new({
        let common = Data::new(common);
        let itools = Data::new(itools);
        let pgpool = Data::new(pgpool);
        let qdrant = Data::new(Arc::new(Mutex::new(qdrant)));

        move || {
            let json_configuration = build_json_configuration();

            App::new()
                .wrap(TracingLogger::default())
                .app_data(common.clone())
                .app_data(itools.clone())
                .app_data(pgpool.clone())
                .app_data(qdrant.clone())
                .app_data(json_configuration)
        }
    })
    .listen(listener)?
    .run();

    Ok(server)
}
