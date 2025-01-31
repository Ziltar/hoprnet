// See the specification for a full description of what this is for, but
// essentially we want to post a 'commitment' to each ticket, that allows
// later verification by giving a 'preimage' of that commitment.
//
// We need to persist this string of commitments in the database, and support
// syncing back and forth with those that have been persisted on chain.
import { debug, Hash, HoprDB, toU8a, u8aConcat, U256 } from '@hoprnet/hopr-utils'

import type { PeerId } from '@libp2p/interface-peer-id'
import { keysPBM } from '@libp2p/crypto/keys'

// NOTE: Workaround until also this file is converted to Rust
// Reason being that all cryptography is now in core-crypto, and core-ethereum cannot depend
// on core (would be a circular dependency)
import { derive_commitment_seed, recover_iterated_hash, iterate_hash } from '../../core/lib/core_crypto.js'

const log = debug('hopr-core-ethereum:commitment')

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000

function searchDBFor(db: HoprDB, channelId: Hash, iteration: number): Promise<Uint8Array | undefined> {
  return db.getCommitment(channelId, iteration)
}

export async function findCommitmentPreImage(db: HoprDB, channelId: Hash): Promise<Hash> {
  let currentCommitment = await db.getCurrentCommitment(channelId)
  let result = await recover_iterated_hash(
    currentCommitment.serialize(),
    (i: number) => searchDBFor(db, channelId, i),
    TOTAL_ITERATIONS,
    DB_ITERATION_BLOCK_SIZE
  )
  if (result == undefined) {
    throw Error(`Could not find preImage. Searching for ${currentCommitment.to_hex()}`)
  }
  return new Hash(result.intermediate)
}

export async function bumpCommitment(db: HoprDB, channelId: Hash, newCommitment: Hash) {
  await db.setCurrentCommitment(channelId, newCommitment)
}

type GetCommitment = () => Promise<Hash>
type SetCommitment = (commitment: Hash) => Promise<string>

async function createCommitmentChain(
  db: HoprDB,
  channelId: Hash,
  initialCommitmentSeed: Uint8Array,
  setChainCommitment: SetCommitment
): Promise<void> {
  const intermediates = await iterate_hash(initialCommitmentSeed, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE)

  await db.storeHashIntermediaries(channelId, intermediates)
  const current = new Hash(intermediates.hash())
  await Promise.all([db.setCurrentCommitment(channelId, current), setChainCommitment(current)])
  log('commitment chain initialized')
}

/**
 * Simple class encapsulating channel information
 * used to generate the initial channel commitment.
 */
export class ChannelCommitmentInfo {
  constructor(
    public readonly chainId: number,
    public readonly contractAddress: string,
    public readonly channelId: Hash,
    public readonly channelEpoch: U256
  ) {}

  /**
   * Generate the initial commitment seed using this channel information and the given
   * private node key.
   * All members need to be specified (non-null).
   * @param peerId Local node ID.
   */
  public createInitialCommitmentSeed(peerId: PeerId): Uint8Array {
    if (peerId.privateKey == null) {
      throw Error('Invalid peerId')
    }

    if (this.channelEpoch == null || this.channelId == null) {
      throw Error('Missing channelEpoch or channelId')
    }

    const channelSeedInfo = u8aConcat(
      this.channelEpoch.serialize(),
      toU8a(this.chainId, 4),
      this.channelId.serialize(),
      new TextEncoder().encode(this.contractAddress)
    )

    return derive_commitment_seed(keysPBM.PrivateKey.decode(peerId.privateKey).Data, channelSeedInfo)
  }
}

export async function initializeCommitment(
  db: HoprDB,
  peerId: PeerId,
  channelInfo: ChannelCommitmentInfo,
  getChainCommitment: GetCommitment,
  setChainCommitment: SetCommitment
) {
  const dbContainsAlready = (await db.getCommitment(channelInfo.channelId, 0)) != undefined
  const chainCommitment = await getChainCommitment()

  if (chainCommitment && dbContainsAlready) {
    try {
      await findCommitmentPreImage(db, channelInfo.channelId) // throws if not found
      return
    } catch (e) {
      log(`Secret is found but failed to find preimage, reinitializing.. ${e.message}`)
    }
  }
  log(`reinitializing (db: ${dbContainsAlready}, chain: ${chainCommitment}})`)
  await createCommitmentChain(
    db,
    channelInfo.channelId,
    channelInfo.createInitialCommitmentSeed(peerId),
    setChainCommitment
  )
}
