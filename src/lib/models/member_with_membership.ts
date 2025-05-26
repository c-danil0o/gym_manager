export interface MemberWithMembership {
	id: number; // Member ID
	card_id: string | null;
	first_name: string;
	last_name: string;
	email: string | null;
	phone: string | null;
	member_created_at: string; // Assuming NaiveDateTime serializes to string

	membership_id: number | null;
	membership_type_name: string | null;
	membership_start_date: string | null;
	membership_end_date: string | null;
	membership_status: string | null;
	remaining_visits: number | null;
}

export interface PaginatedMembersResponse {
	members: MemberWithMembership[];
	total_items: number;
	total_pages: number;
	current_page: number;
	page_size: number;
}
