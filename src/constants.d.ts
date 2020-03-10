type Constants = {
  // Length of hash values generated by the on-chain hash function
  HASH_LENGTH: number

  // Length of signatures that are used on-chain
  SIGNATURE_LENGTH: number

  // Name of the network, e.g. `mainnet` or `testnet`
  NETWORK: string

  // Name of the chain, e.g. `ethereum` or `polkadot`
  CHAIN_NAME: string
}

export default Constants
