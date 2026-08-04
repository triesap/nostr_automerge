use crate::RawEventBytes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AcquisitionSource {
    Relay(String),
    Lan,
    Nearby,
    Import(String),
}

#[derive(Clone, Debug)]
pub(crate) struct AcquiredRawEvent {
    raw: RawEventBytes,
    pub(crate) source: AcquisitionSource,
}

impl AcquiredRawEvent {
    pub(crate) const fn new(raw: RawEventBytes, source: AcquisitionSource) -> Self {
        Self { raw, source }
    }

    pub(crate) fn into_raw(self) -> RawEventBytes {
        self.raw
    }
}

#[cfg(test)]
mod tests {
    use super::{AcquiredRawEvent, AcquisitionSource};
    use crate::evidence::corpus_builder::CorpusBuilder;
    use crate::{ProtocolRevision, RawEventBytes};

    #[test]
    fn prove_acquisition_metadata_has_no_semantic_path() {
        let raw = RawEventBytes::new(
            include_bytes!("../../../../fixtures/v1_draft/nip01/valid_event.json"),
            ProtocolRevision::draft_v1(),
        );
        assert!(raw.is_ok());
        let raw = match raw {
            Ok(raw) => raw,
            Err(_) => return,
        };
        let sources = [
            AcquisitionSource::Relay("wss://relay.example".to_owned()),
            AcquisitionSource::Lan,
            AcquisitionSource::Nearby,
            AcquisitionSource::Import("backup.ndjson".to_owned()),
        ];
        let corpora = sources.map(|source| {
            let acquired = AcquiredRawEvent::new(raw.clone(), source);
            assert!(matches!(
                &acquired.source,
                AcquisitionSource::Relay(_)
                    | AcquisitionSource::Lan
                    | AcquisitionSource::Nearby
                    | AcquisitionSource::Import(_)
            ));
            let mut builder = CorpusBuilder::default();
            builder.ingest_acquired(acquired);
            builder.finish()
        });
        assert!(corpora.windows(2).all(|pair| pair[0] == pair[1]));
    }
}
