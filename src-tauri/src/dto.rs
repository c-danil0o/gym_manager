use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::models::{Member, Membership, MembershipType};

#[derive(Deserialize)]
pub struct MemberPayload {
    pub id: Option<i64>, // Optional ID for updates, required for new members
    pub card_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct MembershipPayload {
    pub member_id: i64, // Member ID to associate with the membership

    pub membership_id: Option<i64>,
    pub membership_type_id: Option<i64>,
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub membership_remaining_visits: Option<i64>,
    pub membership_suspended: Option<bool>,
}

#[derive(Serialize)]
pub struct MemberResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct GetMembersPaginatedPayload {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    pub search_string: Option<String>,
    pub filter_fields: Option<Vec<FilterField>>,
}

#[derive(Deserialize, Debug)]
pub struct FilterField {
  pub field: String, // Field name to filter by
  pub value: String, // Value to filter the field by
}

#[derive(Deserialize, Debug)]
pub struct GetMemberByIdPayload {
    pub id: i64, // Member ID to fetch
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemberWithMembershipUpdate {
    pub id: i64, // Member ID
    pub card_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,

    pub membership_type_id: Option<i64>,

    pub membership_id: Option<i64>,
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub membership_remaining_visits: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedMembersResponse {
    pub data : Vec<MemberInfo>,
    pub total: i64,
    pub total_pages: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Deserialize)]
pub struct NewMembershipTypePayload {
    pub name: String,
    pub duration_days: Option<i64>,
    pub visit_limit: Option<i64>,
    pub enter_by: Option<i64>,
    pub price: f64,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct MembershipResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupPayload {
    pub members: Vec<Member>,
    pub memberships: Vec<Membership>,
    pub membership_types: Vec<MembershipType>,
    // pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct MemberInfo {
    pub id: i64, // Member ID
    pub card_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub name: String, // Full name for convenience
    pub email: Option<String>,
    pub phone: Option<String>,
    pub member_created_at: NaiveDateTime,

    pub membership_type_name: Option<String>,

    pub membership_id: Option<i64>,
    pub membership_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub membership_status: Option<String>,
    pub membership_remaining_visits: Option<i64>,
    pub membership_purchase_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MembershipInfo {
    pub member_id: i64, // Member ID
    pub member_first_name: Option<String>,
    pub member_last_name: Option<String>,

    pub membership_type_id: Option<i64>,
    pub membership_type_name: Option<String>,
    pub membership_type_enter_by: Option<i64>,

    pub membership_id: Option<i64>,
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub membership_status: Option<String>,
    pub membership_remaining_visits: Option<i64>,
    pub membership_purchase_date: Option<NaiveDateTime>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EntryStatus {
    Allowed,
    DeniedNoMembership,
    DeniedMembershipExpired,
    DeniedNoVisitsLeft,
    DeniedAlreadyCheckedIn,
    DeniedAfterHours,
    DeniedMembershipNotActiveYet,
    DeniedMembershipSuspended,
    DeniedMemberNotFound,
    DeniedCardNotAssigned,
    Error
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScanProcessingResult {
    pub status: EntryStatus,
    pub message: String,
    pub member_name: Option<String>,
    pub card_id: Option<String>,
    pub membership_type_name: Option<String>,
    pub membership_end_date: Option<NaiveDate>,
    pub remaining_visits: Option<i64>,
}


#[derive(Deserialize)]
pub struct ScanPayload {
    pub card_id: String,
}


#[derive(Debug, serde::Deserialize)]
pub struct EntryLogSearchParams {
    // Search filters
    pub member_name: Option<String>, // Searches both first_name and last_name
    pub card_id: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<NaiveDate>, // Start date (inclusive)
    pub date_to: Option<NaiveDate>,   // End date (inclusive)

    // Pagination
    pub page: Option<u32>,     // Page number (1-based), defaults to 1
    pub per_page: Option<u32>, // Items per page, defaults to 50, max 500

    // Ordering
    pub order_by: Option<String>, // Field to order by: "entry_time", "member_name", "status", "card_id"
    pub order_direction: Option<String>, // "asc" or "desc", defaults to "desc"
}

// DTO for search results
#[derive(Debug, serde::Serialize)]
pub struct EntryLogSearchResult {
    pub entries: Vec<EntryLogDisplay>,
    pub total_count: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

// DTO for entry log display
#[derive(Debug, serde::Serialize, Clone, FromRow)]
pub struct EntryLogDisplay {
    pub id: i64,
    pub member_id: Option<i64>,
    pub membership_id: Option<i64>,
    pub member_name: Option<String>,
    pub membership_type_name: Option<String>,
    pub card_id: Option<String>,
    pub entry_time: NaiveDateTime,
    pub status: Option<String>,
    pub notes: Option<String>,
}

impl Default for EntryLogSearchParams {
    fn default() -> Self {
        Self {
            member_name: None,
            card_id: None,
            status: None,
            date_from: None,
            date_to: None,
            page: Some(1),
            per_page: Some(50),
            order_by: Some("entry_time".to_string()),
            order_direction: Some("desc".to_string()),
        }
    }
}
