use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct CustomRules {
    pub max_tramway_speed: Option<f64>,
    pub max_subway_speed: Option<f64>,
    pub max_rail_speed: Option<f64>,
    pub max_bus_speed: Option<f64>,
    pub max_ferry_speed: Option<f64>,
    pub max_cable_car_speed: Option<f64>,
    pub max_gondola_speed: Option<f64>,
    pub max_funicular_speed: Option<f64>,
    pub max_coach_speed: Option<f64>,
    pub max_air_speed: Option<f64>,
    pub max_taxi_speed: Option<f64>,
    pub max_other_speed: Option<f64>,
}

pub fn custom_rules(file_path: Option<String>) -> CustomRules {
    if let Some(path) = file_path {
        let f = std::fs::File::open(path).expect("Could not open custom-rules file");
        let d: CustomRules =
            serde_yaml::from_reader(f).expect("custom_rules file is not a valid YAML file");
        log::info!("Load custom rules...ok");
        d
    } else {
        CustomRules {
            ..Default::default()
        }
    }
}

#[test]
fn test_no_custom_rules() {
    let file_path = None;
    let custom_rules = custom_rules(file_path);
    assert_eq!(None, custom_rules.bus_speed);
    assert_eq!(None, custom_rules.air_speed);
}

#[test]
fn test_some_custom_rules() {
    let file_path = Some(String::from("test_data/custom_rules/custom_rules.yml"));
    let custom_rules = custom_rules(file_path);
    assert_eq!(Some(10.), custom_rules.bus_speed);
    assert_eq!(Some(100.5), custom_rules.gondola_speed);
    assert_eq!(None, custom_rules.air_speed);
}

#[test]
#[should_panic(expected = "Could not open custom_rules file")]
fn test_no_file() {
    let file_path = Some(String::from("xxx"));
    custom_rules(file_path);
}

#[test]
#[should_panic(expected = "custom_rules file is not a valid YAML file")]
fn test_bad_file() {
    let file_path = Some(String::from("test_data/custom_rules/bad_custom_rules.yml"));
    custom_rules(file_path);
}
