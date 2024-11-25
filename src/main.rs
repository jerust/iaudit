use iaudit::configuration::setting::get_configuration;
use iaudit::startup::Application;
use iaudit::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (subscriber, _guard) = telemetry::get_subscriber("iaudit", "info", std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let application = Application::build(configuration).await?;

    application.run_until_stopped().await
}
