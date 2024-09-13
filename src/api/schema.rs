// module schema

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomerDetails {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "surname")]
    pub surname: String,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "email")]
    pub email: String,

    #[serde(rename = "mobile")]
    pub mobile: String,
}
