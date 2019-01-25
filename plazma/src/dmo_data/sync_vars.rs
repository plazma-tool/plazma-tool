use serde_yaml;

/// Not using a BTreeMap in preparation for `no_std`.
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncVars {
    pub tracks: Vec<SyncTrack>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncTrack {
    pub name: String,
    pub value: f64,
}

impl Default for SyncVars {
    fn default() -> SyncVars {
        let text = include_str!("../../data/builtin/default_sync_tracks.yml");
        let tracks: Vec<SyncTrack> = serde_yaml::from_str(&text).unwrap();

        SyncVars {
            tracks: tracks,
        }
    }
}

