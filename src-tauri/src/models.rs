use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
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
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupPayload {
    pub members: Vec<Member>,
    pub memberships: Vec<Membership>,
    pub membership_types: Vec<MembershipType>,
    // pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MemberInfo {
    pub id: i64, // Member ID
    pub card_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub member_created_at: NaiveDateTime,

    pub membership_type_name: Option<String>,

    pub membership_id: Option<i64>,
    pub membership_status: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MemberWithMembership {
    pub id: i64, // Member ID
    pub card_id: Option<String>,
    pub short_card_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub member_created_at: NaiveDateTime,

    pub membership_type_id: Option<i64>,
    pub membership_type_name: Option<String>,
    pub membership_type_duration_days: Option<i64>,
    pub membership_type_visit_limit: Option<i64>,
    pub membership_type_enter_by: Option<i64>,
    pub membership_type_price: Option<f64>,

    pub membership_id: Option<i64>,
    pub membership_start_date: Option<NaiveDateTime>,
    pub membership_end_date: Option<NaiveDateTime>,
    pub membership_status: Option<String>,
    pub membership_remaining_visits: Option<i64>,
    pub membership_purchase_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedMembersResponse {
    pub members: Vec<MemberInfo>,
    pub total_items: i64,
    pub total_pages: i64,
    pub current_page: i32,
    pub page_size: i32,
}
