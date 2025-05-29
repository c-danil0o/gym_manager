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
}

#[derive(Serialize)]
pub struct MemberResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct GetMembersPaginatedPayload {
    pub page: Option<i32>,            // 1-indexed page number
    pub page_size: Option<i32>,       // Number of items per page
    pub search_query: Option<String>, // Optional search query
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
    pub members: Vec<MemberInfo>,
    pub total_items: i64,
    pub total_pages: i64,
    pub current_page: i32,
    pub page_size: i32,
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

    pub membership_id: Option<i64>,
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub membership_status: Option<String>,
    pub membership_remaining_visits: Option<i64>,
    pub membership_purchase_date: Option<NaiveDateTime>,
}
