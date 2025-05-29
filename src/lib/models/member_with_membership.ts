export interface MemberInfo {
	id: number; // Member ID
	card_id: string | null;
	first_name: string;
	last_name: string;
	email: string | null;
	phone: string | null;
	member_created_at: string;

	membership_id: number | null;
	membership_type_name: string | null;
	membership_status: string | null;
}

export interface PaginatedMembersResponse {
	members: MemberInfo[];
	total_items: number;
	total_pages: number;
	current_page: number;
	page_size: number;
}

export interface MemberWithMembership {
  id: number;
	card_id: string | null;
	short_card_id: string | null;
	first_name: string;
	last_name: string;
	email: string | null;
	phone: string | null;
	date_of_birth: string | null;
	member_created_at: string;

	membership_id: number | null;
	membership_status: string | null;
	membership_start_date: string | null;
	membership_end_date: string | null;
	membership_remaining_visits: number | null;
	membership_purchase_date: string | null;

	membership_type_id: number | null;
	membership_type_name: string | null;
  membership_type_price: number | null;
  membership_type_duration_days: number | null;
  membership_type_visit_limit: number | null;
  membership_type_enter_by: number | null;
}

export interface MembershipInfo {
  member_id: number;
  member_first_name: string | null;
  member_last_name: string | null;

	membership_id: number | null;
	membership_status: string | null;
	membership_start_date: string | null;
	membership_end_date: string | null;
	membership_remaining_visits: number | null;
	membership_purchase_date: string | null;

	membership_type_id: number | null;
	membership_type_name: string | null;
}
