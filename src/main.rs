use iaudit::configuration::setting;
use iaudit::startup::Application;
use iaudit::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (subscriber, _guard) = telemetry::get_subscriber("iaudit", "info", std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = setting::get_configuration().expect("读取配置文件失败");

    let application = Application::build(configuration).await?;

    application.run_until_stopped().await
}
