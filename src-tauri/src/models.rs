use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Member {
    pub id: i64,
    pub card_id: Option<String>,
    pub short_card_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MembershipType {
    pub id: i64,
    pub name: String,
    pub duration_days: Option<i64>,
    pub visit_limit: Option<i64>,
    pub enter_by: Option<i64>,
    pub price: f64,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Membership {
    pub id: i64,
    pub member_id: i64,
    pub membership_type_id: i64,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub remaining_visits: Option<i64>,
    pub status: String,
    pub purchase_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct EntryLog {
    pub id: i64,
    pub member_id: i64,
    pub membership_id: Option<i64>,
    pub entry_time: NaiveDateTime,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub notes: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct BackupStatus {
    pub id: i64,
    pub last_check_time: NaiveDateTime,
    pub last_successful_upload_time: Option<NaiveDateTime>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
}

// --- API / Command Payloads ---
