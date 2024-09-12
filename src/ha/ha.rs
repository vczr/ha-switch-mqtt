use serde::{Deserialize, Serialize};

pub static HOME_ASSISTANT_TOPIC_HEADER :&'static str = "homeassistant/";

#[derive(Serialize, Deserialize)]
pub struct InitEntity {
    pub name: String ,
    pub config_topic: String,
    pub sw_version: String ,
    pub object_id: String ,
    pub unique_id: String ,
    pub command_topic: String ,
    pub availability_topic: String ,
    pub state_topic: String
}

impl InitEntity{
    pub fn new (name :String ,id :String) -> InitEntity{
        InitEntity {
            name,
            config_topic: std::format!("{}switch/{}/config",HOME_ASSISTANT_TOPIC_HEADER.clone() ,id.clone()),
            sw_version: String::from("1.0"),
            object_id: id.clone(),
            unique_id: id.clone(),
            command_topic: std::format!("{}switch/{}/set",HOME_ASSISTANT_TOPIC_HEADER.clone() ,id.clone()),
            availability_topic: std::format!("{}switch/{}/ava",HOME_ASSISTANT_TOPIC_HEADER.clone() ,id.clone()),
            state_topic: std::format!("{}switch/{}/state",HOME_ASSISTANT_TOPIC_HEADER.clone() ,id.clone()),
        }
    }
}
