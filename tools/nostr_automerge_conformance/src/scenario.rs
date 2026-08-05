use base64::Engine as _;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioInput {
    pub(crate) scenario_schema: String,
    pub(crate) coordinate: String,
    pub(crate) raw_events: Vec<RawScenarioEvent>,
    pub(crate) budget: ScenarioBudget,
    pub(crate) cancel_after: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum RawScenarioEvent {
    Utf8(String),
    Encoded(EncodedRawEvent),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct EncodedRawEvent {
    encoding: RawEncoding,
    data: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RawEncoding {
    Base64,
}

impl RawScenarioEvent {
    pub(crate) fn decode(self) -> Result<Vec<u8>, ScenarioError> {
        match self {
            Self::Utf8(value) => Ok(value.into_bytes()),
            Self::Encoded(value) => base64::engine::general_purpose::STANDARD
                .decode(value.data)
                .map_err(|_| ScenarioError),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioBudget {
    pub(crate) max_bytes: u64,
    pub(crate) max_items: u64,
}

impl ScenarioInput {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, ScenarioError> {
        let value: Self = serde_json::from_slice(bytes).map_err(|_| ScenarioError)?;
        if value.scenario_schema != "nostr_automerge.scenario.v1" || value.raw_events.is_empty() {
            return Err(ScenarioError);
        }
        value
            .coordinate
            .parse::<nostr_automerge::DocumentCoordinate>()
            .map_err(|_| ScenarioError)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScenarioError;

#[cfg(test)]
mod tests {
    use super::ScenarioInput;

    #[test]
    fn define_generic_raw_event_scenario_schema() {
        let json = format!(
            r#"{{"budget":{{"max_bytes":1000,"max_items":100}},"cancel_after":null,"coordinate":"31624:{}:{}","raw_events":["{{}}"],"scenario_schema":"nostr_automerge.scenario.v1"}}"#,
            "11".repeat(32),
            "22".repeat(32)
        );
        assert!(ScenarioInput::parse(json.as_bytes()).is_ok());
        assert!(
            ScenarioInput::parse(json.replace("null", "null,\"unknown\":1").as_bytes()).is_err()
        );
        assert!(
            ScenarioInput::parse(json.replace("scenario.v1", "scenario.v2").as_bytes()).is_err()
        );
    }
}
