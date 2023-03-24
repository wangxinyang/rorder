use abi::reservation_service_server::ReservationServiceServer;
use abi::Config;
use anyhow::Ok;
use anyhow::Result;
use roder_service::RsvpService;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_file("./reservation.yml")?;

    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    println!("ReservationServer listening on: {addr}");

    let svc = RsvpService::from_config(&config).await?;
    let svc = ReservationServiceServer::new(svc);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
