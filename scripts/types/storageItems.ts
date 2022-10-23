export interface StorageItems {
  codeIds: {
    liquidationFilterer?: number
    addressProvider?: number
  }
  addresses: {
    liquidationFilterer?: string
    addressProvider?: string
  }
  execute: {
    addressProviderUpdated?: boolean
  }
}
