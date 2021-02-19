use serde_derive::{Deserialize, Serialize};

use crate::enums::itemtype::{ItemSubType, ItemType};
use crate::enums::medaltier::MedalTier;
use crate::response::utils::prepend_base_url_option;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DisplayPropertiesData {
    pub description: Option<String>,
    pub name: String,

    //https://stackoverflow.com/a/44303505/10232
    #[serde(default)]
    #[serde(rename = "icon", deserialize_with = "prepend_base_url_option")]
    pub icon_path: Option<String>,

    #[serde(rename = "hasIcon")]
    pub has_icon: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryItemDefinitionData {
    #[serde(rename = "hash")]
    pub id: u32,

    #[serde(rename = "displayProperties")]
    pub display_properties: DisplayPropertiesData,

    #[serde(rename = "itemTypeDisplayName")]
    pub item_type_display_name: Option<String>,

    #[serde(rename = "itemTypeAndTierDisplayName")]
    pub item_type_and_tier_display_name: Option<String>,

    #[serde(rename = "itemType")]
    pub item_type: ItemType,

    #[serde(rename = "itemSubType")]
    pub item_sub_type: ItemSubType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActivityDefinitionData {
    #[serde(rename = "hash")]
    pub id: u32,

    #[serde(rename = "displayProperties")]
    pub display_properties: DisplayPropertiesData,

    #[serde(default)]
    #[serde(
        rename = "pgcrImage",
        deserialize_with = "prepend_base_url_option"
    )]
    pub pgcr_image: Option<String>,

    #[serde(rename = "destinationHash")]
    pub destination_hash: u32,

    #[serde(rename = "placeHash")]
    pub place_hash: u32,

    #[serde(rename = "activityTypeHash")]
    pub activity_type_hash: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DestinationDefinitionData {
    #[serde(rename = "hash")]
    pub id: u32,

    #[serde(rename = "displayProperties")]
    pub display_properties: DisplayPropertiesData,

    #[serde(rename = "placeHash")]
    pub place_hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoricalStatsDefinition {
    #[serde(rename = "statId")]
    pub id: String,

    #[serde(rename = "statName")]
    pub name: String,

    #[serde(default, rename = "statDescription")]
    pub description: String,

    #[serde(rename = "iconImage")]
    pub icon_image_path: Option<String>,

    pub weight: i32,

    #[serde(rename = "medalTierHash")]
    pub medal_tier: Option<MedalTier>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaceDefinitionData {
    #[serde(rename = "hash")]
    pub id: u32,

    #[serde(rename = "displayProperties")]
    pub display_properties: DisplayPropertiesData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActivityTypeDefinitionData {
    #[serde(rename = "hash")]
    pub id: u32,

    #[serde(rename = "displayProperties")]
    pub display_properties: DisplayPropertiesData,
}
