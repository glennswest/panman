use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WidgetKind {
    Toggle,
    Slider,
    Sensor,
    Thermostat,
    Status,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WidgetDef {
    pub kind: WidgetKind,
    pub device_id: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub properties: Vec<String>,
    #[serde(default)]
    pub property: Option<String>,
    #[serde(default)]
    pub min: Option<i64>,
    #[serde(default)]
    pub max: Option<i64>,
}
