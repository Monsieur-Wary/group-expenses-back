use crate::helpers;
use group_expenses::configuration;
use std::env;

// Parallel tests on the same env variable causes problems.
#[test]
fn should_use_application_port_env_var_if_specified() {
    helpers::initialize();
    env::set_var("APPLICATION_PORT", "8001");

    let port = 8001;
    env::set_var("APPLICATION_PORT", port.to_string());
    let settings_res = configuration::Settings::new();
    assert!(settings_res.is_ok());
    assert_eq!(port, settings_res.unwrap().application_port());
}

#[test]
fn should_use_the_default_application_port_without_env_var() {
    helpers::initialize();

    env::remove_var("APPLICATION_PORT");
    let settings = configuration::Settings::new();

    assert!(settings.is_ok(), settings.err().unwrap().to_string());
    assert_eq!(
        configuration::Settings::new().unwrap().application_port(),
        settings.unwrap().application_port()
    );
}
