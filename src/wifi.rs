//WIFI
#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

use anyhow::bail;
use anyhow::Result;

use embedded_svc::wifi::ClientConfiguration;
use embedded_svc::wifi::Configuration;
use embedded_svc::wifi::Wifi;
    
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::wifi::WifiWait;

use esp_idf_svc::netif::EspNetifWait;
use esp_idf_svc::netif::EspNetif;

use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::modem::Modem;

use std::time::Duration;
use std::net::Ipv4Addr;


// WIFI
pub fn wifi(
    modem: impl Peripheral<P = Modem> + 'static,
    sysloop: EspSystemEventLoop,
    ssid: &str,
    passwd: &str,
    nvs: esp_idf_svc::nvs::EspDefaultNvsPartition,

) -> Result<Box<EspWifi<'static>>> {
   
    let mut wifi = Box::new(
        EspWifi::new(modem,
                     sysloop.clone(),
                     //None,
                     Some(nvs),
        )?
    );
    
    info!("### Wifi created, about to scan");
    
    let ap_infos = wifi.scan()?;

    let ours = ap_infos
        .into_iter()
        .find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!("### Found configured access point {} on channel {}",
              ssid,
              ours.channel,
        );

        Some(ours.channel)
    } else {
        info!("### Configured access point {} not found during scanning, will go with unknown channel", ssid);
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

    wifi.start()?;

    info!("### Starting wifi...");
    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("### Wifi did not start");
    }

    info!("### Connecting wifi...");
    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("### Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    info!("### Wifi DHCP info: {:?}", ip_info);

    Ok(wifi)
}
