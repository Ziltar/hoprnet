import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command } from '../utils/command'

export default class Tickets extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[], 'shows all tickets']
      },
      api,
      extra
    )
  }

  public name() {
    return 'tickets'
  }

  public description() {
    return 'Displays information about your redeemed and unredeemed tickets'
  }

  public async execute(log): Promise<void> {
    log('finding information about tickets...')
    try {
      const stats: any = await this.api.getTicketStats()

      return log(
        toPaddedString([
          ['Tickets:', ''],
          ['- Pending:', stats.pending],
          ['- Unredeemed:', stats.unredeemed],
          ['- Unredeemed Value:', `${stats.unredeemedValue} xHOPR`],
          ['- Redeemed:', stats.redeemed],
          ['- Redeemed Value:', `${stats.redeemedValue} xHOPR`],
          ['- Losing Tickets:', stats.losingTickets],
          ['- Win Proportion:', stats.winProportion * 100],
          ['- Neglected:', stats.neglected],
          ['- Rejected:', stats.rejected],
          ['- Rejected Value:', `${stats.rejectedValue} xHOPR`]
        ])
      )
    } catch (err) {
      return log(`Unexpected error: ${err.message}`)
    }
  }
}
