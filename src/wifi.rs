#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

use anyhow::Result;

use embedded_svc::wifi::ClientConfiguration;
use embedded_svc::wifi::Configuration;
    
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::wifi::BlockingWifi;

use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::modem::Modem;

//
pub fn wifi(
    modem: impl Peripheral<P = Modem> + 'static,
    sysloop: EspSystemEventLoop,
    ssid: &str,
    passwd: &str,
    nvs: esp_idf_svc::nvs::EspDefaultNvsPartition,
) -> Result<Box<EspWifi<'static>>> {
    let mut esp_wifi = EspWifi::new(modem,
                                    sysloop.clone(),
                                    Some(nvs),
    )?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi,
                                      sysloop,
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    info!("Starting wifi...");
    wifi.start()?;

    info!("Scanning...");
    let ap_infos = wifi.scan()?;

    let ours = ap_infos
        .into_iter()
        .find(|a| a.ssid == ssid);
    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid,
            ours.channel,
        );

        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid,
        );

        None
    };

    wifi.set_configuration(&Configuration::Client(
        ClientConfiguration {
            ssid: ssid.into(),
            password: passwd.into(),
            channel,
            ..Default::default()
        },
    ))?;

    info!("Connecting wifi...");
    wifi.connect()?;

    info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;

    let ip_info = wifi
        .wifi()
        .sta_netif()
        .get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}
