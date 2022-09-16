use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct CustomRules {
    pub tramway_speed: Option<f64>,
    pub subway_speed: Option<f64>,
    pub rail_speed: Option<f64>,
    pub bus_speed: Option<f64>,
    pub ferry_speed: Option<f64>,
    pub cable_car_speed: Option<f64>,
    pub gondola_speed: Option<f64>,
    pub funicular_speed: Option<f64>,
    pub coach_speed: Option<f64>,
    pub air_speed: Option<f64>,
    pub taxi_speed: Option<f64>,
    pub other_speed: Option<f64>,
}

pub fn custom_rules(file_path: Option<String>) -> CustomRules {
    if let Some(path) = file_path {
        let f = std::fs::File::open(path).expect("Could not open custom_rules file");
        let d: CustomRules = serde_yaml::from_reader(f).expect("custom_rules file is not a valid YAML file");
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