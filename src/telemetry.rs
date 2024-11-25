use tracing::subscriber;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::{self, MakeWriter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn get_subscriber<S, W>(
    name: S,
    env_filter: S,
    writer: W,
) -> (impl Subscriber + Send + Sync, WorkerGuard)
where
    S: AsRef<str>,
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // 首先读取RUST_LOG环境变量, 如果存在, 日志级别使用RUST_LOG环境变量的值, 如果不存在, 日志级别使用函数参数env_filter设定的值
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter));

    // 输出到控制台
    let formatting_layer = BunyanFormattingLayer::new(name.as_ref().into(), writer);

    // 输出到文件中
    let file_appender = tracing_appender::rolling::daily("logs", "iaudit.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_target(false)
        .with_ansi(false)
        .with_writer(non_blocking);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(file_layer);

    (subscriber, guard)
}

/// 将订阅器设置为全局默认值, 用于处理所有跨度数据, 该函数只能被调用一次
pub fn init_subscriber<T>(subscriber: T)
where
    T: Subscriber + Send + Sync,
{
    LogTracer::init().expect("Failed to init log tracer");
    subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}
