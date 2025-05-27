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
