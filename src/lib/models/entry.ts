export type EntryStatus =
	| 'Allowed'
	| 'DeniedNoMembership'
	| 'DeniedMembershipExpired'
	| 'DeniedNoVisitsLeft'
	| 'DeniedAlreadyCheckedIn'
	| 'DeniedAfterHours'
	| 'DeniedMembershipNotActiveYet'
	| 'DeniedMembershipSuspended'
	| 'DeniedMemberNotFound'
	| 'DeniedCardNotAssigned'
	| 'Error';

export interface ScanProcessingResult {
	status: EntryStatus;
	message: string;
	member_name: string | null;
	card_id: string | null;
	membership_type_name: string | null;
	membership_end_date: string | null;
	remaining_visits: number | null;
}

export interface EntryLog {
	id: number;
	member_id: number | null;
	membership_id: number | null;
	member_name: string | null;
	membership_type_name: string | null;
	card_id: string | null;
	entry_time: string;
	status: string;
	notes: string | null;
}
