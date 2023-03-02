// CONFIG

// if cfg.toml is wrong it will quietly use default values !!!
#[toml_cfg::toml_config]
pub struct Config {
    #[default("00000000-0000-0000-0000-000000000000")]
    uuid: &'static str,

    #[default("")]
    mqtt_broker_url: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
    #[default("/default_mqtt_topic/")]
    mqtt_topic: &'static str,
    
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_pass: &'static str,

    #[default(0)]
    machine_number: u8,
    #[default("default_name")]
    machine_name: &'static str,
    #[default("esp32_c3_rev3_rust_board")]
    machine_type: &'static str,
    #[default("00:00:00:00:00:00")]
    machine_mac: &'static str,

    #[default(true)]
    flag_debug: bool,
    #[default(false)]
    flag_display: bool,
    #[default(false)]
    flag_show_array_index: bool,
    #[default(false)]
    flag_show_heatmap: bool,
    #[default(false)]
    flag_show_payload: bool,
    #[default(true)]
    flag_measure_duration: bool,
    #[default(false)]
    flag_scan_i2c: bool,
    
    #[default(1000)]
    delay_sleep_duration_ms: u16,
    #[default(100)]
    delay_sleep_after_boot: u16,
}


impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // The `f` value implements the `Write` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        //write!(f, "({}, {})", self.x, self.y)

        f.debug_struct("Config")
            .field("uuid", &self.uuid)
            .field("machine_number", &self.machine_number)
            .field("machine_name", &self.machine_name)
            .field("machine_type", &self.machine_type)
            .finish()

        /*
        write!(f, "uuid: {}, machine_number: {}",
               self.uuid,
               self.machine_number,
        )
        */
    }
}
