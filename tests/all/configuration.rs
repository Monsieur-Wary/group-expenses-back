use group_expenses::configuration;
use std::env;

#[test]
fn should_use_the_default_application_port_without_env_var() {
    env::remove_var("application_port");
    let settings_res = configuration::Settings::new();

    assert!(settings_res.is_ok());
    assert_eq!(
        configuration::DEFAULT_APPLICATION_PORT,
        settings_res.unwrap().application_port()
    );
}

#[test]
fn should_use_application_port_env_var_if_specified() {
    let port = 8001;
    env::set_var("application_port", port.to_string());
    let settings_res = configuration::Settings::new();

    assert!(settings_res.is_ok());
    assert_eq!(port, settings_res.unwrap().application_port());
}
