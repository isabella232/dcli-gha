use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use crate::enums::mode::Mode;
use crate::enums::platform::Platform;
use crate::response::drs::{DestinyResponseStatus, IsDestinyAPIResponse};
use crate::response::utils::str_to_datetime;
use crate::response::utils::{
    property_to_u32_value, property_to_value, standing_default, string_to_i64,
};

pub const MAX_ACTIVITIES_REQUEST_COUNT: i32 = 250;

//https://bungie-net.github.io/multi/operation_get_Destiny2-GetActivityHistory.html#operation_get_Destiny2-GetActivityHistory
#[derive(Serialize, Deserialize, Debug)]
pub struct ActivitiesResponse {
    #[serde(rename = "Response")]
    pub response: Option<ActivitiesResponseData>,

    #[serde(flatten)]
    pub status: DestinyResponseStatus,
}

impl IsDestinyAPIResponse for ActivitiesResponse {
    fn get_status(&self) -> &DestinyResponseStatus {
        &self.status
    }
}

//https://bungie-net.github.io/multi/schema_Destiny-HistoricalStats-DestinyActivityHistoryResults.html#schema_Destiny-HistoricalStats-DestinyActivityHistoryResults
#[derive(Serialize, Deserialize, Debug)]
pub struct ActivitiesResponseData {
    #[serde(rename = "activities")]
    pub activities: Option<Vec<Activity>>,
}

//https://bungie-net.github.io/multi/schema_Destiny-HistoricalStats-DestinyHistoricalStatsPeriodGroup.html#schema_Destiny-HistoricalStats-DestinyHistoricalStatsPeriodGroup
#[derive(Serialize, Deserialize, Debug)]
pub struct Activity {
    #[serde(skip_serializing, deserialize_with = "str_to_datetime")]
    pub period: DateTime<Utc>,

    #[serde(rename = "activityDetails")]
    pub details: DestinyHistoricalStatsActivity,

    //todo: can we collapse these down?
    pub values: ActivityHistoricalStatsValues,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ActivityHistoricalStatsValues {
    #[serde(deserialize_with = "property_to_value")]
    pub assists: f32,

    #[serde(deserialize_with = "property_to_value")]
    pub score: f32,

    #[serde(deserialize_with = "property_to_value")]
    pub kills: f32,

    #[serde(deserialize_with = "property_to_value")]
    pub deaths: f32,

    #[serde(
        rename = "averageScorePerKill",
        deserialize_with = "property_to_value"
    )]
    #[serde(default)]
    pub average_score_per_kill: f32,

    #[serde(
        rename = "averageScorePerLife",
        deserialize_with = "property_to_value"
    )]
    #[serde(default)]
    pub average_score_per_life: f32,

    #[serde(deserialize_with = "property_to_value")]
    pub completed: f32,

    #[serde(
        rename = "opponentsDefeated",
        deserialize_with = "property_to_value"
    )]
    pub opponents_defeated: f32,

    #[serde(deserialize_with = "property_to_value")]
    pub efficiency: f32,

    #[serde(
        rename = "killsDeathsRatio",
        deserialize_with = "property_to_value"
    )]
    pub kills_deaths_ratio: f32,

    #[serde(
        rename = "killsDeathsAssists",
        deserialize_with = "property_to_value"
    )]
    pub kills_deaths_assists: f32,

    #[serde(
        rename = "activityDurationSeconds",
        deserialize_with = "property_to_value"
    )]
    pub activity_duration_seconds: f32,
    //TODO: need to make this an option
    #[serde(deserialize_with = "property_to_u32_value")]
    #[serde(default = "standing_default")]
    pub standing: u32,

    #[serde(deserialize_with = "property_to_value")]
    #[serde(default)]
    pub team: f32,

    #[serde(
        rename = "completionReason",
        deserialize_with = "property_to_value"
    )]
    pub completion_reason: f32,

    #[serde(rename = "startSeconds", deserialize_with = "property_to_value")]
    pub start_seconds: f32,

    #[serde(
        rename = "timePlayedSeconds",
        deserialize_with = "property_to_value"
    )]
    pub time_played_seconds: f32,

    #[serde(rename = "playerCount", deserialize_with = "property_to_value")]
    pub player_count: f32,

    #[serde(rename = "teamScore", deserialize_with = "property_to_value")]
    pub team_score: f32,
}

//https://bungie-net.github.io/multi/schema_Destiny-HistoricalStats-DestinyHistoricalStatsActivity.html#schema_Destiny-HistoricalStats-DestinyHistoricalStatsActivity
#[derive(Serialize, Deserialize, Debug)]
pub struct DestinyHistoricalStatsActivity {
    /// The unique hash identifier of the DestinyActivityDefinition that was played.
    /// (Seems to be the same as director_activity_hash)
    #[serde(rename = "referenceId")]
    pub reference_id: u32,

    /// The unique hash identifier of the DestinyActivityDefinition (Manifest) that was played
    #[serde(rename = "directorActivityHash")]
    pub director_activity_hash: u32,

    /// The unique identifier for this *specific* match that was played.
    ///
    /// This value can be used to get additional data about this activity such
    /// as who else was playing via the GetPostGameCarnageReport endpoint.
    #[serde(rename = "instanceId", deserialize_with = "string_to_i64")]
    pub instance_id: i64,

    pub mode: Mode,

    pub modes: Vec<Mode>, //may need to make Option?

    /// Whether or not the match was a private match
    #[serde(rename = "isPrivate")]
    pub is_private: bool,

    /// The platform the activitity was played on
    #[serde(rename = "membershipType")]
    pub membership_type: Platform,
}
