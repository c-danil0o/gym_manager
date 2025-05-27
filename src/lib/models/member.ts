export interface Member {
  id: number,
  card_id?: string,
  short_card_id?: string,
  first_name: string,
  last_name: string,
  email?: string,
  phone?:string,
  date_of_birth?: string,
  created_at: Date
  updated_at: Date
  is_deleted: boolean,
}
