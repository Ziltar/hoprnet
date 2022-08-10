import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command, type CacheFunctions } from '../utils/command'
import { utils as ethersUtils } from 'ethers'

export default class Channels extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[], 'shows open channels'],
        incoming: [[['constant', 'show incoming only', false]], 'show incoming channels'],
        outgoing: [[['constant', 'show outgoing only', false]], 'show outgoing channels']
      },
      api,
      cache
    )
  }

  public name() {
    return 'channels'
  }

  public description() {
    return 'Lists your currently open channels, optionally show all channels'
  }

  /**
   * Creates the log output for a channel
   * @returns a channel log
   */
  private getChannelLog(channel) {
    return toPaddedString([
      ['Outgoing Channel:', channel.channelId],
      ['To:', channel.peerId],
      ['Status:', channel.status],
      ['Balance:', ethersUtils.formatEther(channel.balance)]
    ])
  }

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , param] = this.assertUsage(query) as [string | undefined, string, boolean | string]
    if (error) return log(error)

    const showAll = param === true
    const showIncoming = param === 'incoming'
    const showOutgoing = param === 'outgoing'

    log('fetching channels...')
    const channelsRes = await this.api.getChannels()
    if (!channelsRes.ok) return log(this.invalidResponse('get channels'))
    const channels = await channelsRes.json()

    if (!showOutgoing) {
      const incomingChannels = channels.incoming.filter((channel) => {
        if (showAll) return true
        return channel.status !== 'Closed'
      })
      if (incomingChannels.length == 0) {
        log(`\nNo channels opened to you.`)
      } else {
        for (const channel of incomingChannels) {
          log(this.getChannelLog(channel))
        }
      }
    }

    if (!showIncoming) {
      const channelsOutgoing = channels.outgoing.filter((channel) => {
        if (showAll) return true
        return channel.status !== 'Closed'
      })
      if (channelsOutgoing.length == 0) {
        log(`\nNo channels opened by you.`)
      } else {
        for (const channel of channelsOutgoing) {
          log(this.getChannelLog(channel))
        }
      }
    }
  }
}
