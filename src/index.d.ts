import { LevelUp } from 'levelup'
import BN from 'bn.js'

import IUtils from './utils'
import Channel from './channel'
import Constructors, { Types } from './types'

export { IUtils, Types, Channel }

declare class HoprCoreConnector {
  private constructor(...props: any[])

  readonly started: boolean
  readonly self: any
  readonly db: LevelUp
  readonly nonce: Promise<number>

    /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   */
  static create(db: LevelUp, keyPair: any, uri?: string): Promise<HoprCoreConnector>

  static readonly utils: IUtils
  static readonly channel : Channel
  static readonly types: Constructors

  start(): Promise<void>

  stop(): Promise<void>

  initOnchainValues(nonce?: number): Promise<void>

  checkFreeBalance(newBalance: any): Promise<void>
}


export default HoprCoreConnector