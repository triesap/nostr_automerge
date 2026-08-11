use base64::Engine as _;
use serde::Deserialize;

const SIGNED_SCENARIO_SCHEMA: &str = "nostr_automerge.signed_scenario.v2";
const REVISION: &str = "draft_2026_08";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignedScenarioInput {
    pub(crate) scenario_schema: String,
    pub(crate) fixture_id: String,
    pub(crate) revision: String,
    pub(crate) coordinate: String,
    pub(crate) raw_events: Vec<EncodedRawEventV2>,
    pub(crate) budget: ScenarioBudget,
    pub(crate) cancel_after: Option<u64>,
    pub(crate) requirements: Vec<String>,
    pub(crate) expected_report: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct EncodedRawEventV2 {
    encoding: RawEncodingV2,
    data: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RawEncodingV2 {
    Utf8,
    Base64,
}

impl SignedScenarioInput {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, ScenarioError> {
        let value: Self = serde_json::from_slice(bytes).map_err(|_| ScenarioError)?;
        if value.scenario_schema != SIGNED_SCENARIO_SCHEMA
            || value.revision != REVISION
            || value.raw_events.is_empty()
            || !is_identifier(&value.fixture_id)
            || value.requirements.is_empty()
            || value.requirements.windows(2).any(|pair| pair[0] >= pair[1])
            || value.requirements.iter().any(|id| !is_requirement(id))
            || !value.expected_report.is_object()
        {
            return Err(ScenarioError);
        }
        value
            .coordinate
            .parse::<nostr_automerge::DocumentCoordinate>()
            .map_err(|_| ScenarioError)?;
        Ok(value)
    }

    pub(crate) fn into_scenario(self) -> ScenarioInput {
        ScenarioInput {
            scenario_schema: "nostr_automerge.scenario.v1".to_owned(),
            coordinate: self.coordinate,
            raw_events: self
                .raw_events
                .into_iter()
                .map(|event| match event.encoding {
                    RawEncodingV2::Utf8 => RawScenarioEvent::Utf8(event.data),
                    RawEncodingV2::Base64 => RawScenarioEvent::Encoded(EncodedRawEvent {
                        encoding: RawEncoding::Base64,
                        data: event.data,
                    }),
                })
                .collect(),
            budget: self.budget,
            cancel_after: self.cancel_after,
        }
    }

    pub(crate) fn with_raw_events(mut self, raw_events: Vec<EncodedRawEventV2>) -> Self {
        self.raw_events = raw_events;
        self
    }
}

impl EncodedRawEventV2 {
    pub(crate) fn decoded(&self) -> Result<Vec<u8>, ScenarioError> {
        match self.encoding {
            RawEncodingV2::Utf8 => Ok(self.data.as_bytes().to_vec()),
            RawEncodingV2::Base64 => base64::engine::general_purpose::STANDARD
                .decode(&self.data)
                .map_err(|_| ScenarioError),
        }
    }
}

fn is_identifier(value: &str) -> bool {
    (3..=128).contains(&value.len())
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || (index > 0 && byte == b'_')
        })
}

fn is_requirement(value: &str) -> bool {
    value.starts_with("NCRDT-")
        && value.len() > 6
        && value[6..]
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
}

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
    use super::{ScenarioInput, SignedScenarioInput};

    #[test]
    fn signed_scenario_v2_schema_rejects_protocol_truth_inputs() {
        let json = format!(
            r#"{{"budget":{{"max_bytes":1000,"max_items":100}},"cancel_after":null,"coordinate":"31624:{}:{}","expected_report":{{}},"fixture_id":"signed_control_001","raw_events":[{{"data":"{{}}","encoding":"utf8"}}],"requirements":["NCRDT-CONF-001"],"revision":"draft_2026_08","scenario_schema":"nostr_automerge.signed_scenario.v2"}}"#,
            "11".repeat(32),
            "22".repeat(32)
        );
        assert!(SignedScenarioInput::parse(json.as_bytes()).is_ok());
        for abstract_field in ["valid", "selected", "accepted", "controls", "changes"] {
            let mutated = json.replacen(
                "\"budget\"",
                &format!("\"{abstract_field}\":true,\"budget\""),
                1,
            );
            assert!(SignedScenarioInput::parse(mutated.as_bytes()).is_err());
        }
    }

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
