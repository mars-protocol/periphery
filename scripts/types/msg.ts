export type InstantiateMsgs =
  | FiltererInstantiateMsg

export interface FiltererInstantiateMsg {
  owner: string
  address_provider: string
}

export interface UpdateOwner {
  owner: string
}
