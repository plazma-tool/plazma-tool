use crate::dmo_data::model::Model;

#[derive(Serialize, Deserialize, Debug)]
pub struct PolygonContext {
    pub models: Vec<Model>,
}

impl Default for PolygonContext {
    fn default() -> PolygonContext {
        PolygonContext {
            models: vec![],
        }
    }
}
